// G5 standalone immediate Metal control; opt-in local reference, never shipped.
import AppKit
import Metal
import QuartzCore
import Darwin

let env = ProcessInfo.processInfo.environment
let enabled = env["RUSTUI_G5_REFERENCE"] == "1"
guard enabled else { fatalError("Set RUSTUI_G5_REFERENCE=1 for this local control") }
let seconds = Double(CommandLine.arguments.dropFirst().first ?? "60")!
let hz = Double(env["RUSTUI_G5_INPUT_HZ"] ?? "10")!
let transaction = env["RUSTUI_G5_TRANSACTION"] == "1"
func tick() -> UInt64 { mach_absolute_time() }
var timebase = mach_timebase_info_data_t(); mach_timebase_info(&timebase)
let factor = Double(timebase.numer) / Double(timebase.denom) / 1e9
let recordLock = NSLock()
var records = [[String:Any]]()
func record(_ r:[String:Any]) { recordLock.lock(); records.append(r); recordLock.unlock() }
func cpu() -> Double { var r = rusage(); getrusage(RUSAGE_SELF, &r); return Double(r.ru_utime.tv_sec+r.ru_stime.tv_sec) + Double(r.ru_utime.tv_usec+r.ru_stime.tv_usec)/1e6 }
final class MetalView:NSView { override func makeBackingLayer() -> CALayer { CAMetalLayer() } }
final class Driver:NSObject,NSApplicationDelegate {
 var window:NSWindow!; var layer:CAMetalLayer!; var queue:MTLCommandQueue!
 var timer:Timer?; var link:CADisplayLink!
 var start:UInt64=0; var startCPU=0.0; var version=0; var text=""; var lastSubmitted=0
 var live=Set<Int>(); var finishing=false
 func applicationDidFinishLaunching(_ n:Notification) {
  window=NSWindow(contentRect:NSRect(x:100,y:100,width:1200,height:800),styleMask:[.titled,.closable],backing:.buffered,defer:false)
  window.title="G5 standalone Metal immediate control"
  let view=MetalView(frame:NSRect(x:0,y:0,width:1200,height:800)); view.wantsLayer=true; window.contentView=view
  layer=view.layer as? CAMetalLayer; layer.device=MTLCreateSystemDefaultDevice()!; layer.pixelFormat = .bgra8Unorm
  layer.framebufferOnly=true; layer.displaySyncEnabled=true; layer.maximumDrawableCount=3; layer.presentsWithTransaction=transaction
  let scale=window.backingScaleFactor; layer.contentsScale=scale; layer.drawableSize=CGSize(width:1200*scale,height:800*scale)
  queue=layer.device!.makeCommandQueue()!
  window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps:true)
  link=view.displayLink(target:self,selector:#selector(display(_:))); link.add(to:.main,forMode:.common)
  DispatchQueue.main.asyncAfter(deadline:.now()+2) {
   self.start=tick(); self.startCPU=cpu(); record(["event":"start","mach":self.start,"numer":timebase.numer,"denom":timebase.denom,"hz":hz,"transaction":transaction,"pool":self.layer.maximumDrawableCount])
   self.timer=Timer.scheduledTimer(withTimeInterval:1/hz,repeats:true) { _ in self.input() }
  }
 }
 @objc func display(_ l:CADisplayLink) { if start>0 && !finishing { record(["event":"cadence","timestamp":l.timestamp,"target":l.targetTimestamp]) } }
 func input() {
  if Double(tick()-start)*factor >= seconds { finish(); return }
  version += 1; if version%2==1 { text.append("x") } else { text.removeLast() }
  record(["event":"handled","version":version,"mach":tick()]); draw()
 }
 func draw() {
  guard version>lastSubmitted,live.count<layer.maximumDrawableCount else { return }
  let v=version; let begin=tick(); let d=layer.nextDrawable(); let end=tick()
  record(["event":"next_drawable","version":v,"ms":Double(end-begin)*factor*1000])
  guard let d=d else { record(["event":"acquire_failed","version":v]); return }
  live.insert(v); record(["event":"acquired","version":v,"inflight":live.count,"mach":end])
  let command=queue.makeCommandBuffer()!; let pass=MTLRenderPassDescriptor(); pass.colorAttachments[0].texture=d.texture
  pass.colorAttachments[0].loadAction = .clear; pass.colorAttachments[0].storeAction = .store
  pass.colorAttachments[0].clearColor=MTLClearColor(red:Double(v&255)/255,green:Double((v>>8)&255)/255,blue:0.35,alpha:1)
  let encoder=command.makeRenderCommandEncoder(descriptor:pass)!; encoder.endEncoding()
  record(["event":"draw","version":v,"mach":tick()]); lastSubmitted=v
  command.addCompletedHandler { b in record(["event":"completed","version":v,"mach":tick(),"gpu_end":b.gpuEndTime]); DispatchQueue.main.async { self.draw() } }
  d.addPresentedHandler { shown in
   record(["event":"presented","version":v,"presented":shown.presentedTime,"host":CACurrentMediaTime(),"mach":tick()])
   DispatchQueue.main.async { self.live.remove(v); self.draw() }
  }
  if transaction { CATransaction.begin(); CATransaction.setDisableActions(true); command.commit(); command.waitUntilScheduled(); d.present(); CATransaction.commit() }
  else { command.present(d); command.commit() }
  record(["event":"committed","version":v,"mach":tick()])
 }
 func finish() {
  guard !finishing else {return}; finishing=true; timer?.invalidate(); link.invalidate()
  let end=tick(); let cpuSeconds=cpu()-startCPU
  DispatchQueue.main.asyncAfter(deadline:.now()+0.5) {
   record(["event":"end","mach":end,"inputs":self.version,"text":self.text,"valid":self.text == (self.version%2==1 ? "x":""),"cpu_s":cpuSeconds])
   recordLock.lock(); let all=records; recordLock.unlock()
   for row in all {let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]); print("G5_REF "+String(data:data,encoding:.utf8)!)}
   NSApp.terminate(nil)
  }
 }
}
let app=NSApplication.shared; app.setActivationPolicy(.regular); let driver=Driver(); app.delegate=driver; app.run()
