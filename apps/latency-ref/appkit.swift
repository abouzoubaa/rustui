// PX-4 reference only. Not a Rust workspace target or a shipped application.
import AppKit
import QuartzCore
import Darwin

func ticks() -> UInt64 { mach_absolute_time() }
var base = mach_timebase_info_data_t()
mach_timebase_info(&base)
var records = [[String: Any]]()
func record(_ row: [String: Any]) { records.append(row) }
final class Text: NSTextView {
    var version = 0
    var painted = 0
    var latest: [Int: UInt64] = [:]
    let marker = CALayer()
    override func keyDown(with event: NSEvent) {
        version += 1
        latest[version] = ticks()
        record(["event":"handled", "version":version, "mach":latest[version]!])
        super.keyDown(with:event)
        // Explicit layer contents change makes the version observable in the
        // layer tree. A display-link callback is a refresh-opportunity proxy,
        // not an observed drawable presentation or proof of scanout.
        var pixel: UInt32 = UInt32(version & 255) | 0xff000000
        let data = Data(bytes:&pixel,count:4)
        let provider = CGDataProvider(data:data as CFData)!
        let image = CGImage(width:1,height:1,bitsPerComponent:8,bitsPerPixel:32,bytesPerRow:4,
                            space:CGColorSpaceCreateDeviceRGB(),bitmapInfo:CGBitmapInfo(rawValue:CGImageAlphaInfo.premultipliedLast.rawValue),provider:provider,decode:nil,shouldInterpolate:false,intent:.defaultIntent)!
        CATransaction.begin(); CATransaction.setDisableActions(true)
        marker.contents = image
        CATransaction.commit()
    }
    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        painted = version
        record(["event":"drawn", "version":painted, "mach":ticks()])
    }
}
final class Driver: NSObject, NSApplicationDelegate {
    var window: NSWindow!
    var text: Text!
    var link: CADisplayLink!
    var timer: Timer?
    var measured: UInt64 = 0
    var count = 0
    var reported = Set<Int>()
    let seconds = Double(CommandLine.arguments.dropFirst().first ?? "60")!
    func applicationDidFinishLaunching(_ notification: Notification) {
        window = NSWindow(contentRect:NSRect(x:100,y:100,width:1200,height:800),styleMask:[.titled,.closable,.resizable],backing:.buffered,defer:false)
        window.title = "PX-4 AppKit NSTextView reference"
        let scroll = NSScrollView(frame:NSRect(x:0,y:0,width:1200,height:800))
        text = Text(frame:scroll.bounds); text.isRichText = false
        text.font = NSFont.monospacedSystemFont(ofSize:16,weight:.regular)
        text.wantsLayer = true; text.marker.frame = NSRect(x:1100,y:10,width:20,height:20)
        text.layer!.addSublayer(text.marker)
        scroll.documentView = text; window.contentView = scroll
        window.makeKeyAndOrderFront(nil); window.makeFirstResponder(text)
        NSApp.activate(ignoringOtherApps:true)
        link = window.contentView!.displayLink(target:self,selector:#selector(display(_:)))
        link.add(to:.main,forMode:.common)
        DispatchQueue.main.asyncAfter(deadline:.now()+2) { self.measured = ticks(); self.timer = Timer.scheduledTimer(withTimeInterval:0.1,repeats:true) { _ in self.input() } }
    }
    func input() {
        if Double(ticks()-measured)*Double(base.numer)/Double(base.denom)/1e9 >= seconds { finish(); return }
        count += 1
        let character = count % 2 == 1 ? "x" : "\u{7f}"
        let event = NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:window.windowNumber,context:nil,characters:character,charactersIgnoringModifiers:character,isARepeat:false,keyCode:count % 2 == 1 ? 7:51)!
        text.keyDown(with:event)
    }
    @objc func display(_ link: CADisplayLink) {
        let version = text.painted
        guard version > 0, !reported.contains(version) else { return }
        reported.insert(version)
        record(["event":"display_opportunity", "version":version, "mach":ticks(), "timestamp":link.timestamp, "target_timestamp":link.targetTimestamp])
    }
    func finish() {
        timer?.invalidate();link.invalidate()
        let header:[String:Any] = ["event":"reference", "method":"NSTextView.draw followed by first CADisplayLink callback; marker layer contents changed with input; refresh-opportunity proxy, NOT observed scanout", "numer":base.numer,"denom":base.denom,"duration_s":seconds,"inputs":count,"reported_hz":window.screen?.maximumFramesPerSecond ?? 0]
        for row in [header]+records { let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]); print("LATENCY_REF "+String(data:data,encoding:.utf8)!) }
        NSApp.terminate(nil)
    }
}
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let driver = Driver(); app.delegate = driver
app.run()
