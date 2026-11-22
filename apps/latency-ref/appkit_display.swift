// G5 opt-in local AppKit control: observed marker pixels + WindowServer display time.
import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreVideo
import QuartzCore
import Darwin
let env=ProcessInfo.processInfo.environment
guard env["RUSTUI_G5_REFERENCE"] == "1" else { fatalError("Set RUSTUI_G5_REFERENCE=1") }
guard CGPreflightScreenCaptureAccess() else { fatalError("Screen recording access is required; no callback proxy is substituted") }
let seconds=Double(CommandLine.arguments.dropFirst().first ?? "60")!
let inputHz=Double(env["RUSTUI_G5_INPUT_HZ"] ?? "10")!
var tb=mach_timebase_info_data_t(); mach_timebase_info(&tb)
func tick()->UInt64 { mach_absolute_time() }
let secondsPerTick=Double(tb.numer)/Double(tb.denom)/1e9
let logLock=NSLock(); var records=[[String:Any]]()
func record(_ r:[String:Any]) { logLock.withLock { records.append(r) } }
func snapshot() -> [[String:Any]] { logLock.withLock { records } }
final class Text:NSTextView {
 var version=0; let marker=CALayer()
 override func keyDown(with e:NSEvent) { version += 1; record(["event":"handled","version":version,"mach":tick()]); super.keyDown(with:e) }
 override func draw(_ r:NSRect) {
  super.draw(r)
  // Publish the version only after this text version has actually drawn.
  var rgba=[UInt8]()
  for column in 0..<26 {
   if column==0 || column==25 {rgba += [255,0,0,255]}
   else {let bit=column<=8 ? ((0xa5>>(8-column))&1):((version>>(24-column))&1);let c:UInt8=bit==1 ? 255:0;rgba += [c,c,c,255]}
  }
  let data=Data(rgba);let provider=CGDataProvider(data:data as CFData)!
  let image=CGImage(width:26,height:1,bitsPerComponent:8,bitsPerPixel:32,bytesPerRow:104,space:CGColorSpaceCreateDeviceRGB(),bitmapInfo:CGBitmapInfo(rawValue:CGImageAlphaInfo.premultipliedLast.rawValue),provider:provider,decode:nil,shouldInterpolate:false,intent:.defaultIntent)!
  CATransaction.begin();CATransaction.setDisableActions(true);marker.contents=image;CATransaction.commit()
  record(["event":"draw","version":version,"mach":tick()])
 }
}
final class Driver:NSObject,NSApplicationDelegate {
 var window:NSWindow!;var text:Text!;var capture:VersionCapture!;var link:CADisplayLink!
 var timer:Timer?;var start:UInt64=0;var count=0;var finishing=false
 func applicationDidFinishLaunching(_ n:Notification) {
  window=NSWindow(contentRect:NSRect(x:100,y:100,width:1200,height:800),styleMask:[.titled,.closable],backing:.buffered,defer:false)
  window.title="G5 AppKit observed-display control"
  let scroll=NSScrollView(frame:NSRect(x:0,y:0,width:1200,height:800));text=Text(frame:scroll.bounds)
  text.isRichText=false;text.font=NSFont.monospacedSystemFont(ofSize:16,weight:.regular)
  text.wantsLayer=true;text.marker.frame=NSRect(x:1000,y:10,width:104,height:32);text.marker.magnificationFilter = .nearest;text.layer!.addSublayer(text.marker)
  scroll.documentView=text;window.contentView=scroll;window.makeKeyAndOrderFront(nil);window.makeFirstResponder(text);NSApp.activate(ignoringOtherApps:true)
  link=scroll.displayLink(target:self,selector:#selector(display(_:)));link.add(to:.main,forMode:.common)
  Task {do {
   self.capture=VersionCapture(emit:record);try await self.capture.start(window:self.window)
   for warm in 1...6 {DispatchQueue.main.asyncAfter(deadline:.now()+Double(warm)*0.1){self.sendEdit()}}
   DispatchQueue.main.asyncAfter(deadline:.now()+2) {
    self.count=0;self.start=tick();record(["event":"start","mach":self.start,"numer":tb.numer,"denom":tb.denom,"input_hz":inputHz,"method":"marker pixels in ScreenCaptureKit frame; WindowServer displayTime"])
    self.timer=Timer.scheduledTimer(withTimeInterval:1/inputHz,repeats:true) {_ in self.input()}
   }
  } catch {fputs("G5 AppKit display capture failed: \(error)\n",stderr);exit(2)}}
 }
 @objc func display(_ l:CADisplayLink) {if start>0 && !finishing {record(["event":"cadence","timestamp":l.timestamp,"target":l.targetTimestamp])}}
 func input() {
  if Double(tick()-start)*secondsPerTick>=seconds {finish();return}
  sendEdit()
 }
 func sendEdit() {
  count += 1;let ch=count%2==1 ? "x":"\u{7f}"
  let e=NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:window.windowNumber,context:nil,characters:ch,charactersIgnoringModifiers:ch,isARepeat:false,keyCode:count%2==1 ? 7:51)!
  text.keyDown(with:e)
 }
 func finish() {
  guard !finishing else{return};finishing=true;timer?.invalidate();link.invalidate();let end=tick()
  DispatchQueue.main.asyncAfter(deadline:.now()+0.5) {Task {await self.capture.stop();record(["event":"end","mach":end,"inputs":self.count,"text":self.text.string,"valid":self.text.string == (self.count%2==1 ? "x":"")]);let all=snapshot();for row in all {let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]);print("G5_REF "+String(data:data,encoding:.utf8)!)};NSApp.terminate(nil)}}
 }
}
let app=NSApplication.shared;app.setActivationPolicy(.regular);let driver=Driver();app.delegate=driver;app.run()
