// Shared opt-in Cocoa experiment for the Lab, bare GPUI, and NSTextView.
#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#import <mach/mach_time.h>

typedef NS_ENUM(uint8_t, WarmMode) { WarmOff, WarmFourFrames, Warm250Milliseconds };
typedef struct {
    uint64_t mach;
    const char *kind;
    double first;
    double second;
    double third;
} Sample;

static const double lagPeriod = 0.004;
static const double quietDelay = 0.250;
static Sample samples[65536];
static size_t sampleCount, dropped;
static WarmMode mode;
static BOOL monitorLag, observeScreen, supported;
static CFRunLoopTimerRef lagTimer, idleTimer;
static double nextLag, lastScreenUpdate, deadline;
static unsigned refreshes;
static NSScreen *activeScreen;
static CFRunLoopTimerRef make_timer(double interval, CFRunLoopTimerCallBack callback);

API_AVAILABLE(macos(14.0))
@interface G5WarmDisplayLink : NSObject
@property(nonatomic, strong) CADisplayLink *link;
@property(nonatomic, weak) NSView *view;
@property(nonatomic) NSInteger preferredRate;
- (void)step:(CADisplayLink *)link;
@end
static G5WarmDisplayLink *observer API_AVAILABLE(macos(14.0));

static void record(const char *kind, double first, double second, double third) {
    if (sampleCount == sizeof(samples) / sizeof(samples[0])) {
        dropped++;
        return;
    }
    samples[sampleCount++] = (Sample){ mach_absolute_time(), kind, first, second, third };
}

static void sample_screen(NSScreen *screen) {
    if (!observeScreen || !screen) return;
    if (@available(macOS 12.0, *)) {
        double timestamp = screen.lastDisplayUpdateTimestamp;
        if (timestamp > 0 && timestamp != lastScreenUpdate) {
            lastScreenUpdate = timestamp;
            record("screen_update", timestamp, CACurrentMediaTime(), 0);
        }
    }
}

static void pause_link(void) {
    if (@available(macOS 14.0, *)) {
        if (!observer.link || observer.link.paused) return;
        observer.link.paused = YES;
        record("paused", refreshes, CACurrentMediaTime(), deadline);
    }
}

@implementation G5WarmDisplayLink
- (void)step:(CADisplayLink *)link {
    refreshes++;
    record("link_tick", link.timestamp, link.targetTimestamp, refreshes);
    if (mode == WarmFourFrames && refreshes == 4) pause_link();
}
@end

static void idle_tick(CFRunLoopTimerRef timer, void *unused) {
    (void)unused;
    double left = deadline - CACurrentMediaTime();
    if (left > 0) {
        CFRunLoopTimerSetNextFireDate(timer, CFAbsoluteTimeGetCurrent() + left);
    } else {
        pause_link();
        CFRunLoopTimerSetNextFireDate(timer, CFAbsoluteTimeGetCurrent() + 3600);
    }
}

static void lag_tick(CFRunLoopTimerRef timer, void *unused) {
    (void)timer; (void)unused;
    double now = CACurrentMediaTime();
    double late = now - nextLag;
    if (late > 0.002 && monitorLag) record("main_late", nextLag, now, late);
    // Keep the phase of the 4 ms timer and skip fire times already elapsed.
    nextLag += lagPeriod * (1 + floor(fmax(0, late) / lagPeriod));
    sample_screen(activeScreen);
}

static CFRunLoopTimerRef make_timer(double interval, CFRunLoopTimerCallBack callback) {
    CFRunLoopTimerRef timer = CFRunLoopTimerCreate(NULL, CFAbsoluteTimeGetCurrent() + interval,
                                                  interval, 0, 0, callback, NULL);
    CFRunLoopAddTimer(CFRunLoopGetMain(), timer, kCFRunLoopCommonModes);
    return timer;
}

static void finish(void) {
    if (lagTimer) { CFRunLoopTimerInvalidate(lagTimer); CFRelease(lagTimer); }
    if (idleTimer) { CFRunLoopTimerInvalidate(idleTimer); CFRelease(idleTimer); }
    if (@available(macOS 14.0, *)) [observer.link invalidate];
    for (size_t i = 0; i < sampleCount; i++) {
        Sample s = samples[i];
        fprintf(stderr, "G5_L5 {\"event\":\"%s\",\"mach\":%llu,\"values\":[%.9f,%.9f,%.9f]}\n",
                s.kind, (unsigned long long)s.mach, s.first, s.second, s.third);
    }
    fprintf(stderr, "G5_L5 {\"event\":\"dropped\",\"count\":%zu}\n", dropped);
}

void g5_keep_warm_configure(void) {
    const char *value = getenv("RUSTUI_G5_KEEP_WARM");
    if (value && strcmp(value, "4") == 0) mode = WarmFourFrames;
    if (value && strcmp(value, "250ms") == 0) mode = Warm250Milliseconds;
    value = getenv("RUSTUI_MAIN_LAG");
    monitorLag = value && strcmp(value, "1") == 0;
    value = getenv("RUSTUI_G5_SCK_CADENCE");
    observeScreen = value && strcmp(value, "1") == 0;
    if (mode == WarmOff && !monitorLag && !observeScreen) return;
    if (@available(macOS 14.0, *)) supported = YES;
    if (mode != WarmOff && !supported) record("unsupported", mode, 0, 0);
    atexit(finish);
    if (monitorLag || observeScreen) {
        [NSNotificationCenter.defaultCenter addObserverForName:NSApplicationDidFinishLaunchingNotification
                                                        object:nil queue:nil usingBlock:^(NSNotification *notification) {
            (void)notification;
            activeScreen = NSApp.keyWindow.screen ?: NSScreen.mainScreen;
            nextLag = CACurrentMediaTime() + lagPeriod;
            lagTimer = make_timer(lagPeriod, lag_tick);
        }];
    }
}

void g5_keep_warm_input(NSEvent *event, uint64_t version) {
    if (mode == WarmOff && !monitorLag && !observeScreen) return;
    NSWindow *window = event.window ?: NSApp.keyWindow;
    if (!window) return;
    NSScreen *screen = window.screen;
    if (screen != activeScreen) {
        activeScreen = screen;
        lastScreenUpdate = 0;
        if (@available(macOS 12.0, *)) {
            if (observeScreen) {
                record("screen_limits", screen.maximumFramesPerSecond,
                       screen.minimumRefreshInterval, screen.maximumRefreshInterval);
            }
        }
    }
    if (@available(macOS 12.0, *)) {
        sample_screen(screen);
        if (version && observeScreen) {
            record("key_screen", version, screen.maximumFramesPerSecond,
                   screen.lastDisplayUpdateTimestamp);
        }
    }
    if (!supported || mode == WarmOff) return;
    if (@available(macOS 14.0, *)) {
        NSView *view = window.contentView;
        if (!view || !screen) return;
        if (!observer) observer = [G5WarmDisplayLink new];
        if (observer.view != view) {
            [observer.link invalidate];
            observer.view = view;
            observer.link = [view displayLinkWithTarget:observer selector:@selector(step:)];
            observer.link.paused = YES;
            observer.preferredRate = 0;
            [observer.link addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
        }
        NSInteger rate = screen.maximumFramesPerSecond;
        if (rate <= 0) { record("rate_unavailable", version, 0, 0); return; }
        if (rate != observer.preferredRate) {
            observer.link.preferredFrameRateRange = CAFrameRateRangeMake(rate, rate, rate);
            observer.preferredRate = rate;
            record("pinned", rate, rate, rate);
        }
        refreshes = 0;
        deadline = CACurrentMediaTime() + quietDelay;
        observer.link.paused = NO;
        if (mode == Warm250Milliseconds) {
            if (!idleTimer) idleTimer = make_timer(3600, idle_tick);
            CFRunLoopTimerSetNextFireDate(idleTimer, CFAbsoluteTimeGetCurrent() + quietDelay);
        }
    }
}
