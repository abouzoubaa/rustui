import AppKit
import Foundation

// Window-local, precise NSEvents only. Start the measured minute on the first gesture.
struct Sample {
    let time, x, y, scale: Double
    let phase, momentum: UInt
    var whole: Bool {
        [x * scale, y * scale].allSatisfy { abs($0 - $0.rounded()) < 1e-7 }
    }
}

final class Recorder: NSView {
    var samples = [Sample]()
    var begin: TimeInterval?
    var timer: Timer?
    let output: URL
    let label = NSTextField(labelWithString: "Scroll here using the trackpad. Include slow scrolls and flings.")
    init(output: URL) {
        self.output = output
        super.init(frame: NSRect(x: 0, y: 0, width: 750, height: 420))
        samples.reserveCapacity(24000)
        label.frame = NSRect(x: 30, y: 190, width: 690, height: 40)
        label.alignment = .center
        addSubview(label)
    }
    required init?(coder: NSCoder) { fatalError() }
    override func scrollWheel(with e: NSEvent) {
        guard e.hasPreciseScrollingDeltas, let window else { return }
        if begin == nil {
            begin = e.timestamp
            label.stringValue = "Recording for 60 seconds. Scroll naturally in this window."
            let timer = Timer(timeInterval: 60, repeats: false) { [weak self] _ in self?.finish() }
            RunLoop.main.add(timer, forMode: .common)
            self.timer = timer
        }
        guard let begin, e.timestamp - begin < 60 else { finish(); return }
        samples.append(Sample(time: e.timestamp - begin, x: e.scrollingDeltaX, y: e.scrollingDeltaY,
                              scale: window.backingScaleFactor, phase: e.phase.rawValue, momentum: e.momentumPhase.rawValue))
    }
    func finish() {
        var csv = "elapsed_s,delta_x_pt,delta_y_pt,scale,delta_x_px,delta_y_px,phase_bits,momentum_bits,whole\n"
        for s in samples {
            csv += "\(s.time),\(s.x),\(s.y),\(s.scale),\(s.x*s.scale),\(s.y*s.scale),\(s.phase),\(s.momentum),\(s.whole ? 1 : 0)\n"
        }
        do { try csv.write(to: output, atomically: true, encoding: .utf8) }
        catch { fputs("Capture write failed: \(error)\n", stderr); exit(1) }
        print("Recorded \(samples.count) precise events")
        NSApp.terminate(nil)
    }
}
guard CommandLine.arguments.count == 2 else { exit(2) }
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let recorder = Recorder(output: URL(fileURLWithPath: CommandLine.arguments[1]))
let window = NSWindow(contentRect: recorder.bounds, styleMask: [.titled, .closable], backing: .buffered, defer: false)
window.title = "RustUI trackpad capture"
window.contentView = recorder
window.center()
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
let timeout = Timer(timeInterval: 600, repeats: false) { _ in
    if recorder.begin == nil { fputs("No physical scroll received\n", stderr); exit(3) }
}
RunLoop.main.add(timeout, forMode: .common)
app.run()
