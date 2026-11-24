// Opt-in local L1 instrumentation. No hook, event posting or capture by default.
#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#import <CoreGraphics/CoreGraphics.h>
#import <objc/runtime.h>
#import <mach/mach_time.h>
#import <dlfcn.h>
typedef struct { uint64_t version; double hardware; uint64_t handler; uint64_t kind; int64_t window; uint64_t cg_timestamp; } NativeInfo;
static NativeInfo current;
static BOOL consumed;
static IMP original;
static NSMutableArray<NSDictionary *> *records;
static char version_key;
static NSMutableDictionary<NSNumber *, NSNumber *> *dragVersions;
static const uint64_t magic=0x4c31000000000000ULL;
static BOOL enabled(void){return [NSProcessInfo.processInfo.environment[@"RUSTUI_L1_NATIVE"] isEqualToString:@"1"];}
int g5_native_take(unsigned kind,NativeInfo *out){if(!enabled()||!current.version||consumed||current.kind!=kind)return 0;*out=current;consumed=YES;return 1;}
static void native_send(id self,SEL selector,NSEvent *event){
 CGEventRef cg=(event.type==NSEventTypeKeyDown||event.type==NSEventTypeMouseMoved)?event.CGEvent:NULL;
 uint64_t data=cg?(uint64_t)CGEventGetIntegerValueField(cg,kCGEventSourceUserData):0;
 const BOOL pointer=event.type==NSEventTypeMouseMoved || event.type==NSEventTypeLeftMouseDragged;
 NSNumber *postedVersion=pointer?objc_getAssociatedObject(event,&version_key):nil;
 const uint64_t pointerTag=pointer?(uint64_t)event.eventNumber:0;
 if(event.type==NSEventTypeLeftMouseDragged && !postedVersion) {
  // The run loop may reconstruct the posted NSEvent. Its timestamp survives
  // reconstruction even when AppKit replaces eventNumber and object identity.
  postedVersion=dragVersions[@(llround(event.timestamp*1000000.0))];
 }
 if(postedVersion) {
  data=magic|postedVersion.unsignedLongLongValue;
  [dragVersions removeObjectForKey:@(llround(event.timestamp*1000000.0))];
 }
 if ((pointerTag&0xffff0000ULL)==0x4c310000ULL) data=magic|(pointerTag&0xffffULL);
 BOOL measured=(data&0xffff000000000000ULL)==magic && (event.type==NSEventTypeKeyDown || pointer);
 NativeInfo previous=current;BOOL previous_consumed=consumed;
 if(measured){current=(NativeInfo){data&0xffffffffULL,event.timestamp,mach_absolute_time(),event.type==NSEventTypeKeyDown?0:1,event.windowNumber,cg?CGEventGetTimestamp(cg):0};consumed=NO;}
 ((void(*)(id,SEL,NSEvent *))original)(self,selector,event);
 if(measured){[records addObject:@{@"event":@"native",@"version":@(current.version),@"hardware_s":@(current.hardware),@"handler_mach":@(current.handler),@"handler_end_mach":@(mach_absolute_time()),@"cg_timestamp_ns":@(current.cg_timestamp),@"window":@(current.window),@"kind":@(current.kind),@"consumed":@(consumed)}];current=previous;consumed=previous_consumed;}
}
int g5_native_post(unsigned kind,uint64_t version,double x,double y){
 if(!enabled())return 0;
 NSWindow *window=NSApp.keyWindow ?: NSApp.mainWindow; if(!window)return 0;
 if(kind==0){
  if(!CGPreflightPostEventAccess())return 0;
  CGKeyCode key=version%2?7:51;
  CGEventRef down=CGEventCreateKeyboardEvent(NULL,key,true),up=CGEventCreateKeyboardEvent(NULL,key,false);
  if(!down||!up){if(down)CFRelease(down);if(up)CFRelease(up);return 0;}
  CGEventSetIntegerValueField(down,kCGEventSourceUserData,(int64_t)(magic|version));
  CGEventPostToPid(getpid(),down);CGEventPostToPid(getpid(),up);CFRelease(down);CFRelease(up);return 1;
 }
 NSView *view=window.contentView;NSPoint p=NSMakePoint(x,view.isFlipped?y:view.bounds.size.height-y);
 p=[view convertPoint:p toView:nil];
 if(kind==1){
  NSPoint screen=[window convertPointToScreen:p];
  CGPoint location=CGPointMake(screen.x,CGDisplayBounds(CGMainDisplayID()).size.height-screen.y);
  CGEventRef move=CGEventCreateMouseEvent(NULL,kCGEventMouseMoved,location,kCGMouseButtonLeft);
  if(!move)return 0;
  CGEventSetIntegerValueField(move,kCGEventSourceUserData,(int64_t)(magic|version));
  CGEventPostToPid(getpid(),move);CFRelease(move);return 1;
 }
 // Explicitly name the target window: CGEventPostToPid drag events may reach
 // NSApp as window 0, bypassing the view and the actual drag state.
 NSEventType type=kind==1?NSEventTypeMouseMoved:kind==2?NSEventTypeLeftMouseDragged:kind==3?NSEventTypeLeftMouseDown:NSEventTypeLeftMouseUp;
 NSEvent *event=[NSEvent mouseEventWithType:type location:p modifierFlags:0 timestamp:CACurrentMediaTime()
                               windowNumber:window.windowNumber context:nil eventNumber:(NSInteger)(0x4c310000ULL|version)
                                 clickCount:1 pressure:kind==4?0:1];
 if(!event)return 0;
 if(kind==2) {
  dragVersions[@(llround(event.timestamp*1000000.0))]=@(version);
  objc_setAssociatedObject(event,&version_key,@(version),OBJC_ASSOCIATION_RETAIN_NONATOMIC);
 }
 [NSApp postEvent:event atStart:NO];
 return 1;
}
static void flush(void){
 mach_timebase_info_data_t b;mach_timebase_info(&b);fprintf(stderr,"L1_NATIVE {\"event\":\"clock\",\"numer\":%u,\"denom\":%u}\n",b.numer,b.denom);
 for(NSDictionary *r in records){NSData *d=[NSJSONSerialization dataWithJSONObject:r options:NSJSONWritingSortedKeys error:nil];fprintf(stderr,"L1_NATIVE %s\n",[[[NSString alloc]initWithData:d encoding:NSUTF8StringEncoding]UTF8String]);}
}
__attribute__((constructor)) static void install(void){
 if(!enabled())return;records=[NSMutableArray new];dragVersions=[NSMutableDictionary new];atexit(flush);
 [NSNotificationCenter.defaultCenter addObserverForName:NSApplicationDidFinishLaunchingNotification object:nil queue:nil usingBlock:^(NSNotification*n){
  (void)n;Class cls=object_getClass(NSApp);SEL sel=@selector(sendEvent:);Method method=class_getInstanceMethod(cls,sel);original=method_getImplementation(method);
  if(!class_addMethod(cls,sel,(IMP)native_send,method_getTypeEncoding(method)))method_setImplementation(class_getInstanceMethod(cls,sel),(IMP)native_send);
  if([NSProcessInfo.processInfo.environment[@"RUSTUI_L1_LAB"]isEqualToString:@"1"] && [NSProcessInfo.processInfo.environment[@"RUSTUI_G5_PIXEL_CAPTURE"]isEqualToString:@"1"]){void(*start)(void)=dlsym(RTLD_DEFAULT,"g5_pixels_start");if(start)start();}
 }];
}
