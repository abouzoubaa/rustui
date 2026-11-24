// The same ScreenCaptureKit decoder for both app launch routes.
import AppKit
import Foundation

@main struct Observer {
 @MainActor static func main() async throws {
  NSApplication.shared.setActivationPolicy(.prohibited)
  let root = URL(fileURLWithPath: CommandLine.arguments[1])
  let windowFile = root.appendingPathComponent("window")
  let deadline = Date().addingTimeInterval(15)
  while !FileManager.default.fileExists(atPath: windowFile.path) {
   guard Date() < deadline else { throw CocoaError(.fileReadNoSuchFile) }
   try await Task.sleep(nanoseconds: 10_000_000)
  }
  let fields = try String(contentsOf: windowFile, encoding: .utf8).split(separator: " ")
  guard fields.count == 3, let id = UInt32(fields[0]),
        let width = Int(fields[1]), let height = Int(fields[2]) else {
   throw CocoaError(.fileReadCorruptFile)
  }
  let cadence: ((UInt64, Int) -> Void)?
  if ProcessInfo.processInfo.environment["RUSTUI_G5_SCK_CADENCE"] == "1" {
   cadence = { display, status in
    print("G5_SCK_CADENCE {\"display_mach\":\(display),\"status\":\(status)}")
   }
  } else {
   cadence = nil
  }
  let capture = VersionCapture(cadence: cadence) { row in
   guard let data = try? JSONSerialization.data(withJSONObject: row, options: [.sortedKeys]) else { return }
   print("G5_PIXELS " + String(decoding: data, as: UTF8.self))
  }
  try await capture.start(id: id, width: width, height: height)
  try Data().write(to: root.appendingPathComponent("ready"))
  while !FileManager.default.fileExists(atPath: root.appendingPathComponent("stop").path) {
   try await Task.sleep(nanoseconds: 50_000_000)
  }
  await capture.stop()
 }
}
