// G5 opt-in local AppKit control: observed marker pixels + WindowServer display time.
import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreVideo
import QuartzCore
import Darwin
let env=ProcessInfo.processInfo.environment
guard env["RUSTUI_G5_REFERENCE"] == "1" else { fatalError("Set RUSTUI_G5_REFERENCE=1") }
guard env["RUSTUI_L1_CAPTURE_DIR"] != nil || CGPreflightScreenCaptureAccess() else { fatalError("Screen recording access is required; no callback proxy is substituted") }
let native = env["RUSTUI_L1_NATIVE"] == "1"
let physical = env["RUSTUI_L1_PHYSICAL"] == "1"
let documentURL = env["RUSTUI_G5_L7_DOCUMENT"].map { URL(fileURLWithPath: $0) }
let document = try documentURL.map { try String(contentsOf: $0, encoding: .utf8) } ?? ""
let nativePost = dlsym(UnsafeMutableRawPointer(bitPattern: -2), "g5_native_post")
let nativeClaim = dlsym(UnsafeMutableRawPointer(bitPattern: -2), "g5_native_claim_key")
let physicalCounter = dlsym(UnsafeMutableRawPointer(bitPattern: -2), "g5_native_physical")
if native && (nativePost == nil || nativeClaim == nil || physicalCounter == nil) {
 fatalError("Native input helper is missing")
}
func countPhysical(_ arm: Int32) -> UInt64 {
 unsafeBitCast(physicalCounter!, to: (@convention(c) (Int32) -> UInt64).self)(arm)
}
let seconds=Double(CommandLine.arguments.dropFirst().first ?? "60")!
let inputHz=Double(env["RUSTUI_G5_INPUT_HZ"] ?? "10")!
var tb=mach_timebase_info_data_t(); mach_timebase_info(&tb)
func tick()->UInt64 { mach_absolute_time() }
let secondsPerTick=Double(tb.numer)/Double(tb.denom)/1e9
func processCPU() -> Double {
 var usage = rusage()
 precondition(getrusage(RUSAGE_SELF, &usage) == 0)
 return Double(usage.ru_utime.tv_sec) + Double(usage.ru_stime.tv_sec)
     + Double(usage.ru_utime.tv_usec + usage.ru_stime.tv_usec) / 1_000_000
}
let logLock=NSLock(); var records=[[String:Any]]()
func record(_ r:[String:Any]) { logLock.withLock { records.append(r) } }
func snapshot() -> [[String:Any]] { logLock.withLock { records } }
final class Text:NSTextView {
 var version=0; let marker=CALayer()
 override func keyDown(with e:NSEvent) {
  let handled = tick()
  if native {
   let claim = unsafeBitCast(nativeClaim!, to: (@convention(c) () -> UInt64).self)
   let input = claim()
   super.keyDown(with:e)
   if input != 0 {
    version = Int(input)
    needsDisplay = true
    record(["event":"handled","version":version,"mach":handled])
   }
  } else {
   version += 1
   record(["event":"handled","version":version,"mach":handled])
   super.keyDown(with:e)
  }
 }
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
 var cpuAtStart = 0.0
 func applicationDidFinishLaunching(_ n:Notification) {
  window=NSWindow(contentRect:NSRect(x:100,y:100,width:1200,height:800),styleMask:[.titled,.closable],backing:.buffered,defer:false)
  window.title="G5 AppKit observed-display control"
  let scroll=NSScrollView(frame:NSRect(x:0,y:0,width:1200,height:800));text=Text(frame:scroll.bounds)
  text.isRichText=false;text.font=NSFont.monospacedSystemFont(ofSize:16,weight:.regular)
  text.wantsLayer=true;text.marker.frame=NSRect(x:1000,y:10,width:104,height:32);text.marker.magnificationFilter = .nearest
  if documentURL != nil {
   let size = Double(env["RUSTUI_G5_L7_FONT_SIZE"] ?? "") ?? 13
   if let family = env["RUSTUI_G5_L7_FONT"], let font = NSFont(name: family, size: size) {
    text.font = font
   } else {
    text.font = NSFont.monospacedSystemFont(ofSize: size, weight: .regular)
   }
   let viewport = NSView(frame: scroll.bounds)
   scroll.frame = NSRect(x:32,y:68,width:1136,height:700)
   scroll.hasVerticalScroller = true
   text.frame = NSRect(origin:.zero,size:scroll.contentSize)
   text.isVerticallyResizable = true
   text.autoresizingMask = [.width]
   text.textContainer?.widthTracksTextView = true
   text.textContainer?.containerSize = NSSize(width:scroll.contentSize.width, height:.greatestFiniteMagnitude)
   text.textContainerInset = .zero
   text.string = document
   // Match the Lab editor’s 1.5× line height in the comparative viewport.
   let paragraph = NSMutableParagraphStyle()
   paragraph.minimumLineHeight = text.font!.pointSize * 1.5
   paragraph.maximumLineHeight = paragraph.minimumLineHeight
   let attrs: [NSAttributedString.Key:Any] = [.font:text.font!, .paragraphStyle:paragraph]
   text.textStorage?.setAttributes(attrs, range:NSRange(location:0,length:(document as NSString).length))
   text.typingAttributes = attrs
   text.setSelectedRange(NSRange(location:2,length:0))
   scroll.documentView = text
   viewport.addSubview(scroll)
   let markerView = NSView(frame: NSRect(x:1000,y:758,width:104,height:32))
   markerView.wantsLayer = true
   text.marker.frame = markerView.bounds
   markerView.layer!.addSublayer(text.marker)
   viewport.addSubview(markerView)
   window.contentView = viewport
  } else {
   text.layer!.addSublayer(text.marker)
   scroll.documentView = text
   window.contentView = scroll
  }
  window.makeKeyAndOrderFront(nil);window.makeFirstResponder(text);NSApp.activate(ignoringOtherApps:true)
  if !native || env["RUSTUI_G5_REFRESH"] == "1" {
   link=scroll.displayLink(target:self,selector:#selector(display(_:)));link.add(to:.main,forMode:.common)
  }
  Task {do {
   self.capture=VersionCapture(emit:record);try await self.capture.start(window:self.window)
   if !native {
    for warm in 1...6 {DispatchQueue.main.asyncAfter(deadline:.now()+Double(warm)*0.1){self.sendEdit()}}
   }
   DispatchQueue.main.asyncAfter(deadline:.now()+2) {
    if documentURL != nil, let layout = self.text.layoutManager, let container = self.text.textContainer {
     let inset = self.text.textContainerOrigin
     let visible = self.text.visibleRect.offsetBy(dx:-inset.x,dy:-inset.y)
     let range = layout.glyphRange(forBoundingRect:visible, in:container)
     var rows = 0
     layout.enumerateLineFragments(forGlyphRange:range) { _, _, _, _, _ in rows += 1 }
     record(["event":"document","visible_rows":rows,"visible_layout_glyphs":range.length,
             "font":self.text.font!.fontName,"point_size":self.text.font!.pointSize,
             "content_width":self.window.contentView!.bounds.width,
             "content_height":self.window.contentView!.bounds.height,
             "viewport_width":visible.width,"viewport_height":visible.height,
             "lines":document.split(separator:"\n",omittingEmptySubsequences:false).count])
    }
    self.count=0;self.cpuAtStart=processCPU();self.start=tick();record(["event":"start","mach":self.start,"numer":tb.numer,"denom":tb.denom,"input_hz":inputHz,"method":"marker pixels in ScreenCaptureKit frame; WindowServer displayTime"])
    if physical {
     _ = countPhysical(1)
     DispatchQueue.main.asyncAfter(deadline:.now()+seconds) {self.finish()}
    } else {
     self.timer=Timer.scheduledTimer(withTimeInterval:1/inputHz,repeats:true) {_ in self.input()}
    }
   }
  } catch {fputs("G5 AppKit display capture failed: \(error)\n",stderr);exit(2)}}
 }
 @objc func display(_ l:CADisplayLink) {if start>0 && !finishing {record(["event":"cadence","timestamp":l.timestamp,"target":l.targetTimestamp])}}
 func input() {
  if Double(tick()-start)*secondsPerTick>=seconds {finish();return}
  sendEdit()
 }
 func sendEdit() {
  count += 1
  if native {
   let post=unsafeBitCast(nativePost!,to:(@convention(c) (UInt32,UInt64,Double,Double)->Int32).self)
   precondition(post(0,UInt64(count),0,0) == 1,"native key post failed")
   return
  }
  let ch=count%2==1 ? "x":"\u{7f}"
  let e=NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:window.windowNumber,context:nil,characters:ch,charactersIgnoringModifiers:ch,isARepeat:false,keyCode:count%2==1 ? 7:51)!
  text.keyDown(with:e)
 }
 func finish() {
  guard !finishing else{return};finishing=true;timer?.invalidate();link?.invalidate();let end=tick();let cpu=processCPU()-cpuAtStart
  if physical { count=Int(countPhysical(0)) }
  DispatchQueue.main.asyncAfter(deadline:.now()+0.5) {
   Task {
    await self.capture.stop()
    var expected = document
    if self.count % 2 == 1 { expected.insert("x", at:expected.index(expected.startIndex,offsetBy:documentURL == nil ? 0 : 2)) }
    record(["event":"end","mach":end,"cpu_s":cpu,"inputs":self.count,
            "text":documentURL == nil ? self.text.string : "",
            "valid":self.text.string == expected])
    for row in snapshot() {
     let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys])
     fputs("G5_REF "+String(data:data,encoding:.utf8)!+"\n",stderr)
    }
    NSApp.terminate(nil)
   }
  }
 }
}
let app=NSApplication.shared;app.setActivationPolicy(.regular);let driver=Driver();app.delegate=driver;app.run()
