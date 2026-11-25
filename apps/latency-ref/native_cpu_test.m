#import <AppKit/AppKit.h>
#import <mach/mach_time.h>
#import <dlfcn.h>
#include <assert.h>
#import "native_cpu.h"

int main(void) {
    @autoreleasepool {
        setenv("RUSTUI_G5_L8_CPU", "1", 1);
        cpu_startup();
        g5_cpu_boundary(0);

        mach_timebase_info_data_t timebase;
        assert(mach_timebase_info(&timebase) == KERN_SUCCESS);
        uint64_t start = mach_absolute_time();
        uint64_t cpu_begin = cpu_handler_begin(1);
        const struct timespec nap = {.tv_nsec = 15 * 1000 * 1000};
        assert(nanosleep(&nap, NULL) == 0);
        uint64_t elapsed_cpu = cpu_handler_end(cpu_begin);
        uint64_t elapsed_wall = (mach_absolute_time() - start) * timebase.numer / timebase.denom;
        assert(elapsed_wall >= 10 * 1000 * 1000 && elapsed_cpu < elapsed_wall);

        g5_cpu_boundary(1);
        for (int phase = 0; phase < 2; phase++) {
            bool main_present = false;
            for (unsigned i = 0; i < boundaries[phase].count; i++) {
                if (boundaries[phase].threads[i].tid == main_tid)
                    main_present = true;
            }
            assert(main_present && boundaries[phase].power_valid && boundaries[phase].unread == 0);
        }
        assert(boundaries[1].mach > boundaries[0].mach);
        assert(boundaries[1].user_us + boundaries[1].sys_us >=
               boundaries[0].user_us + boundaries[0].sys_us);
    }
}
