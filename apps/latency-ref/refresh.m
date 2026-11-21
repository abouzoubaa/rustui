// G5 local reference observer. No presents and no preferred-rate request.
#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
@interface G5Refresh : NSObject
@property(nonatomic,strong) CADisplayLink *link;
@property(nonatomic,strong) NSMutableArray<NSArray<NSNumber *> *> *samples;
- (void)step:(CADisplayLink *)link;
@end
@implementation G5Refresh
- (void)step:(CADisplayLink *)link { [self.samples addObject:@[@(link.timestamp),@(link.targetTimestamp)]]; }
@end
static G5Refresh *observer;
void g5_refresh_start(void) {
 if (![[NSProcessInfo.processInfo.environment objectForKey:@"RUSTUI_G5_REFRESH"] isEqualToString:@"1"]) return;
 observer=[G5Refresh new];observer.samples=[NSMutableArray new];
 NSView *view=NSApp.keyWindow.contentView;
 if (!view) { fprintf(stderr,"G5_REFRESH_ERROR no key-window view\n");return; }
 observer.link=[view displayLinkWithTarget:observer selector:@selector(step:)];
 [observer.link addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
}
void g5_refresh_stop(void) {
 [observer.link invalidate];
 NSMutableString *output=[NSMutableString new];
 for (NSArray<NSNumber *> *r in observer.samples) {
  [output appendFormat:@"G5_REFRESH {\"timestamp\":%.9f,\"target\":%.9f}\n",r[0].doubleValue,r[1].doubleValue];
 }
 fputs(output.UTF8String,stderr);observer=nil;
}
