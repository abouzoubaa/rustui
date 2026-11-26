// SPDX-License-Identifier: Apache-2.0
// Separate G-R2 allocation capture. Count Rust System allocation entry points
// plus other callers of these symbols on the macOS main thread.
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>

static uint64_t calls;

uint64_t rustui_gr2_main_allocations(void) { return calls; }

static void count(void) {
    if (pthread_main_np()) ++calls;
}

static void *measure_malloc(size_t n) { count(); return malloc(n); }
static void *measure_calloc(size_t n, size_t size) { count(); return calloc(n, size); }
static void *measure_realloc(void *p, size_t n) { count(); return realloc(p, n); }
static void *measure_aligned(size_t align, size_t size) { count(); return aligned_alloc(align, size); }
static int measure_posix(void **p, size_t align, size_t size) {
    count();
    return posix_memalign(p, align, size);
}

#define OBSERVE(replacement, original) \
    __attribute__((used, section("__DATA,__interpose"))) \
    static const struct { const void *to; const void *from; } hook_##original = \
        {(const void *)replacement, (const void *)original}
OBSERVE(measure_malloc, malloc);
OBSERVE(measure_calloc, calloc);
OBSERVE(measure_realloc, realloc);
OBSERVE(measure_aligned, aligned_alloc);
OBSERVE(measure_posix, posix_memalign);
