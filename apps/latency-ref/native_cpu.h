/* Opt-in diagnostics linked only into the latency reference helper. */
#import <mach/mach.h>
#import <mach/mach_vm.h>
#import <mach/thread_status.h>
#include <dispatch/dispatch.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>
#include <time.h>
#include <sys/resource.h>
#if defined(__arm64e__)
#include <ptrauth.h>
#endif

static bool cpu_probe_enabled;
static bool wait_probe_enabled;
static atomic_uint_fast64_t waiting_version;
static atomic_bool waiting_done;
static atomic_uint_fast64_t idle_sources[2];
static pthread_t wait_probe;
static dispatch_semaphore_t wait_ready;
static mach_port_t main_port;
static uint64_t main_tid;
static uintptr_t stack_low, stack_high;
static NSWindow *cpu_window;
enum {CPU_THREAD_LIMIT = 512, WAIT_LIMIT = 8192, WAIT_FRAMES = 36};

struct thread_sample {
    uint64_t tid, user_us, system_us;
    char name[64];
};
static struct {
    uint64_t mach;
    uint64_t user_us, sys_us;
    double epoch_ms;
    uint64_t caret_ticks, appkit_caret_draws;
    task_power_info_data_t power;
    bool power_valid, active, key;
    struct thread_sample threads[CPU_THREAD_LIMIT];
    unsigned count, unread;
} boundaries[2];

struct wait_sample {uint64_t version, mach; unsigned depth; uintptr_t pc[WAIT_FRAMES];};
static struct wait_sample blocks[WAIT_LIMIT];
static unsigned block_count, block_dropped, sample_failed;
static struct {uint64_t version, cpu_ns;} handler_times[8192];
static unsigned handler_count, handler_dropped;
static struct {unsigned runnable, waiting, other;} wait_activity[8192];

static void cpu_record_handler(uint64_t version, uint64_t cpu_ns) {
    if (handler_count == sizeof(handler_times) / sizeof(handler_times[0])) {
        handler_dropped++;
        return;
    }
    handler_times[handler_count++] = (typeof(handler_times[0])){version, cpu_ns};
}

static uint64_t microseconds(time_value_t v) {
    return (uint64_t)v.seconds * 1000000 + v.microseconds;
}

static uint64_t timeval_us(struct timeval v) {
    return (uint64_t)v.tv_sec * 1000000 + v.tv_usec;
}

void g5_cpu_idle_prepare(int unfocused) {
    if (!cpu_probe_enabled) return;
    cpu_window=NSApp.keyWindow ?: NSApp.mainWindow;
    if (unfocused) [NSApp deactivate];
}

void g5_cpu_note_idle_source(unsigned source) {
    if (cpu_probe_enabled && source < 2)
        atomic_fetch_add_explicit(&idle_sources[source], 1, memory_order_relaxed);
}

void g5_cpu_boundary(unsigned phase) {
    if (!cpu_probe_enabled || phase > 1) return;
    typeof(boundaries[0]) *snapshot = &boundaries[phase];
    thread_act_array_t threads;
    mach_msg_type_number_t count = 0;
    struct rusage usage;
    snapshot->mach = mach_absolute_time();
    snapshot->epoch_ms = (double)clock_gettime_nsec_np(CLOCK_REALTIME) / 1000000.0;
    if (getrusage(RUSAGE_SELF, &usage) == 0) {
        snapshot->user_us = timeval_us(usage.ru_utime);
        snapshot->sys_us = timeval_us(usage.ru_stime);
    }
    mach_msg_type_number_t size = TASK_POWER_INFO_COUNT;
    snapshot->power_valid = task_info(mach_task_self(), TASK_POWER_INFO,
        (task_info_t)&snapshot->power, &size) == KERN_SUCCESS;
    snapshot->caret_ticks = atomic_load_explicit(&idle_sources[0], memory_order_relaxed);
    snapshot->appkit_caret_draws = atomic_load_explicit(&idle_sources[1], memory_order_relaxed);
    snapshot->active = NSApp.isActive;
    snapshot->key = (cpu_window ?: NSApp.keyWindow).isKeyWindow;
    if (task_threads(mach_task_self(), &threads, &count) != KERN_SUCCESS) {
        snapshot->unread++;
        return;
    }
    for (mach_msg_type_number_t i = 0; i < count; i++) {
        thread_act_t thread = threads[i];
        thread_basic_info_data_t times;
        thread_identifier_info_data_t info;
        mach_msg_type_number_t times_count = THREAD_BASIC_INFO_COUNT;
        mach_msg_type_number_t id_count = THREAD_IDENTIFIER_INFO_COUNT;
        if (snapshot->count < CPU_THREAD_LIMIT &&
            thread_info(thread, THREAD_IDENTIFIER_INFO, (thread_info_t)&info, &id_count) == KERN_SUCCESS &&
            thread_info(thread, THREAD_BASIC_INFO, (thread_info_t)&times, &times_count) == KERN_SUCCESS) {
            struct thread_sample *row = &snapshot->threads[snapshot->count++];
            row->tid = info.thread_id;
            row->user_us = microseconds(times.user_time);
            row->system_us = microseconds(times.system_time);
            pthread_t pthread = pthread_from_mach_thread_np(thread);
            if (pthread) pthread_getname_np(pthread, row->name, sizeof(row->name));
        } else {
            snapshot->unread++;
        }
        mach_port_deallocate(mach_task_self(), thread);
    }
    vm_deallocate(mach_task_self(), (vm_address_t)threads, sizeof(thread_act_t) * count);
}

#if defined(__aarch64__)
static uintptr_t strip_return(uintptr_t address) {
#if defined(__arm64e__)
    return (uintptr_t)ptrauth_strip((void*)address, ptrauth_key_return_address);
#else
    // arm64 may unwind arm64e system libraries with signed return addresses.
    return address & UINT64_C(0x0000ffffffffffff);
#endif
}

static void sample_block(uint64_t version, uint64_t ticks) {
    if (block_count == WAIT_LIMIT) {
        block_dropped++;
        return;
    }
    struct wait_sample *sample = &blocks[block_count];
    sample->depth = 0;
    if (thread_suspend(main_port) != KERN_SUCCESS) {
        sample_failed++;
        return;
    }
    if (atomic_load_explicit(&waiting_version, memory_order_acquire) != version) {
        thread_resume(main_port);
        return;
    }
    arm_thread_state64_t state;
    mach_msg_type_number_t count = ARM_THREAD_STATE64_COUNT;
    if (thread_get_state(main_port, ARM_THREAD_STATE64, (thread_state_t)&state, &count) == KERN_SUCCESS) {
        sample->version = version;
        sample->mach = ticks;
        sample->depth = 0;
        uintptr_t pc = arm_thread_state64_get_pc(state);
        if (pc) sample->pc[sample->depth++] = pc;
        uintptr_t lr = strip_return(arm_thread_state64_get_lr(state));
        if (lr && lr != pc) sample->pc[sample->depth++] = lr - 1;
        uintptr_t fp = arm_thread_state64_get_fp(state);
        while (sample->depth < WAIT_FRAMES && fp >= stack_low && fp <= stack_high - 16 && (fp & 15) == 0) {
            uintptr_t frame[2];
            mach_vm_size_t actual = 0;
            if (mach_vm_read_overwrite(mach_task_self(), fp, sizeof(frame),
                (mach_vm_address_t)&frame, &actual) != KERN_SUCCESS || actual != sizeof(frame))
                break;
            uintptr_t ret = strip_return(frame[1]);
            if (!ret) break;
            if (!sample->depth || sample->pc[sample->depth-1] != ret - 1)
                sample->pc[sample->depth++] = ret - 1;
            if (frame[0] <= fp) break;
            fp = frame[0];
        }
    } else {
        sample_failed++;
    }
    thread_resume(main_port);
    if (sample->depth > 0) block_count++;
}
#endif

static void *observe_waits(void *arg) {
    (void)arg;
    pthread_setname_np("G5 handler wait observer");
    const struct timespec nap = {.tv_nsec = 50000};
    while (!atomic_load_explicit(&waiting_done, memory_order_relaxed)) {
        uint64_t version = atomic_load_explicit(&waiting_version, memory_order_acquire);
        if (!version) {
            dispatch_semaphore_wait(wait_ready, DISPATCH_TIME_FOREVER);
            continue;
        }
#if defined(__aarch64__)
        uint64_t now = mach_absolute_time();
        thread_basic_info_data_t info;
        mach_msg_type_number_t size = THREAD_BASIC_INFO_COUNT;
        if (thread_info(main_port, THREAD_BASIC_INFO, (thread_info_t)&info, &size) == KERN_SUCCESS &&
            atomic_load_explicit(&waiting_version, memory_order_acquire) == version) {
            if (version < sizeof(wait_activity) / sizeof(wait_activity[0])) {
                typeof(wait_activity[0]) *activity = &wait_activity[version];
                if (info.run_state == TH_STATE_WAITING) activity->waiting++;
                else if (info.run_state == TH_STATE_RUNNING) activity->runnable++;
                else activity->other++;
            }
            if (info.run_state == TH_STATE_WAITING)
                sample_block(version, now);
        }
#endif
        nanosleep(&nap, NULL);
    }
    return NULL;
}

static void cpu_flush(void) {
    if (!cpu_probe_enabled) return;
    if (wait_probe_enabled) {
        atomic_store_explicit(&waiting_done, true, memory_order_relaxed);
        dispatch_semaphore_signal(wait_ready);
        pthread_join(wait_probe, NULL);
        mach_port_deallocate(mach_task_self(), main_port);
    }
    fprintf(stderr,"G5_L8_CPU {\"event\":\"main\",\"tid\":%llu,\"wait_dropped\":%u,\"wait_failed\":%u,\"handler_dropped\":%u}\n",
            main_tid, block_dropped, sample_failed, handler_dropped);
    for (unsigned i = 0; i < handler_count; i++)
        fprintf(stderr,"G5_L8_CPU {\"event\":\"handler\",\"version\":%llu,\"cpu_ns\":%llu}\n",
                handler_times[i].version, handler_times[i].cpu_ns);
    for (unsigned phase = 0; phase < 2; phase++) {
        typeof(boundaries[0]) *s = &boundaries[phase];
        if (!s->mach) continue;
        fprintf(stderr, "G5_L8_CPU {\"event\":\"boundary\",\"phase\":%u,\"mach\":%llu,\"epoch_ms\":%.6f,\"user_us\":%llu,\"sys_us\":%llu,\"key\":%s,\"active\":%s,\"unread\":%u,\"power_valid\":%s,\"interrupt_wakeups\":%llu,\"platform_idle_wakeups\":%llu,\"timer_wakeups_1\":%llu,\"timer_wakeups_2\":%llu,\"caret_ticks\":%llu,\"appkit_caret_draws\":%llu}\n",
            phase, s->mach, s->epoch_ms, s->user_us, s->sys_us, s->key?"true":"false", s->active?"true":"false",
            s->unread, s->power_valid?"true":"false", s->power.task_interrupt_wakeups,
            s->power.task_platform_idle_wakeups, s->power.task_timer_wakeups_bin_1, s->power.task_timer_wakeups_bin_2, s->caret_ticks, s->appkit_caret_draws);
        for (unsigned i = 0; i < s->count; i++) {
            struct thread_sample *t = &s->threads[i];
            NSData *data = [NSJSONSerialization dataWithJSONObject:@{@"event":@"thread",@"phase":@(phase),
                @"tid":@(t->tid),@"user_us":@(t->user_us),@"sys_us":@(t->system_us),
                @"name":[NSString stringWithUTF8String:t->name] ?: @""}
                options:0 error:NULL];
            fprintf(stderr, "G5_L8_CPU %.*s\n", (int)data.length, (const char *)data.bytes);
        }
    }
    for (unsigned i = 1; i < sizeof(wait_activity) / sizeof(wait_activity[0]); i++) {
        const typeof(wait_activity[0]) *activity = &wait_activity[i];
        if (activity->waiting || activity->runnable || activity->other)
            fprintf(stderr, "G5_L8_CPU {\"event\":\"wait_activity\",\"version\":%u,\"waiting\":%u,\"runnable\":%u,\"other\":%u}\n",
                    i, activity->waiting, activity->runnable, activity->other);
    }
    for (unsigned i = 0; i < block_count; i++) {
        struct wait_sample *s = &blocks[i];
        fprintf(stderr,"G5_L8_STACK %llu %llu",s->version,s->mach);
        for (unsigned f=0; f<s->depth; f++) {
            uintptr_t pc = s->pc[f]; Dl_info image;
            if (dladdr((const void*)pc, &image) && image.dli_fname) {
                fprintf(stderr,"\t%s|%#llx|%s",image.dli_fname,
                    (unsigned long long)(pc-(uintptr_t)image.dli_fbase),image.dli_sname ?: "");
            }
        }
        fputc('\n',stderr);
    }
}

static void cpu_startup(void) {
    cpu_probe_enabled = getenv("RUSTUI_G5_L8_CPU") && strcmp(getenv("RUSTUI_G5_L8_CPU"),"1")==0;
    if (!cpu_probe_enabled) return;
    pthread_threadid_np(NULL, &main_tid);
    wait_probe_enabled = getenv("RUSTUI_G5_L8_WAIT_STACKS") && strcmp(getenv("RUSTUI_G5_L8_WAIT_STACKS"),"1")==0;
    if (wait_probe_enabled) {
        wait_ready = dispatch_semaphore_create(0);
        main_port = mach_thread_self();
        stack_high = (uintptr_t)pthread_get_stackaddr_np(pthread_self());
        stack_low = stack_high - pthread_get_stacksize_np(pthread_self());
        if (pthread_create(&wait_probe, NULL, observe_waits, NULL) != 0) {
            wait_probe_enabled = false;
            mach_port_deallocate(mach_task_self(), main_port);
        }
    }
    atexit(cpu_flush);
}

static uint64_t cpu_handler_begin(uint64_t version) {
    if (!cpu_probe_enabled) return 0;
    uint64_t cpu = clock_gettime_nsec_np(CLOCK_THREAD_CPUTIME_ID);
    if (wait_probe_enabled) {
        atomic_store_explicit(&waiting_version, version, memory_order_release);
        dispatch_semaphore_signal(wait_ready);
    }
    return cpu;
}

static uint64_t cpu_handler_end(uint64_t started) {
    if (!cpu_probe_enabled) return 0;
    uint64_t end = clock_gettime_nsec_np(CLOCK_THREAD_CPUTIME_ID);
    if (wait_probe_enabled) atomic_store_explicit(&waiting_version, 0, memory_order_release);
    return end >= started ? end-started : 0;
}
