// Native pixel validation only. Timing measurements run separately without capture/OCR.
import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreVideo
import Vision
import ImageIO
import UniformTypeIdentifiers

final class Writer: NSObject, SCStreamOutput, @unchecked Sendable {
    let raw: FileHandle
    let meta: FileHandle
    var offset: UInt64 = 0
    var index = 0
    init(_ folder: URL) throws {
        FileManager.default.createFile(atPath: folder.appendingPathComponent("frames.bgra").path, contents: nil)
        FileManager.default.createFile(atPath: folder.appendingPathComponent("frames.jsonl").path, contents: nil)
        raw = try FileHandle(forWritingTo: folder.appendingPathComponent("frames.bgra"))
        meta = try FileHandle(forWritingTo: folder.appendingPathComponent("frames.jsonl"))
    }
    func stream(_ stream: SCStream, didOutputSampleBuffer sample: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen, let image = CMSampleBufferGetImageBuffer(sample),
              let attachments = CMSampleBufferGetSampleAttachmentsArray(sample, createIfNecessary: false) as? [[SCStreamFrameInfo: Any]],
              let value = attachments.first?[.status] as? Int,
              SCFrameStatus(rawValue: value) == .complete else { return }
        CVPixelBufferLockBaseAddress(image, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(image, .readOnly) }
        let stride = CVPixelBufferGetBytesPerRow(image), height = CVPixelBufferGetHeight(image), width = CVPixelBufferGetWidth(image)
        guard let address = CVPixelBufferGetBaseAddress(image) else { return }
        let data = Data(bytes: address, count: stride * height)
        let record: [String: Any] = ["frame": index, "epoch_ms": (Date().timeIntervalSince1970 - CMClockGetTime(CMClockGetHostTimeClock()).seconds + CMSampleBufferGetPresentationTimeStamp(sample).seconds) * 1000, "callback_ms":Date().timeIntervalSince1970*1000,
                                   "width": width, "height": height, "stride": stride, "offset": offset, "size": data.count]
        do {
            try raw.write(contentsOf: data)
            try meta.write(contentsOf: JSONSerialization.data(withJSONObject: record) + Data([10]))
            offset += UInt64(data.count); index += 1
        } catch { fputs("capture write failed: \(error)\n", stderr) }
    }
}

func makeImage(_ data: Data, _ frame: [String: Any]) -> CGImage? {
    let provider = CGDataProvider(data: data as CFData)!
    return CGImage(width: frame["width"] as! Int, height: frame["height"] as! Int,
                   bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: frame["stride"] as! Int,
                   space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue).union(.byteOrder32Little),
                   provider: provider, decode: nil, shouldInterpolate: false, intent: .defaultIntent)
}

func number(_ text: String, _ pattern: String) -> Int? {
    let regex = try! NSRegularExpression(pattern: pattern)
    let ns = text as NSString
    guard let match = regex.firstMatch(in: text, range: NSRange(location: 0, length: ns.length)), match.numberOfRanges > 1 else { return nil }
    return Int(ns.substring(with: match.range(at: 1)))
}

func median(_ values: [Double]) -> Double {
    let sorted = values.sorted(); return sorted[sorted.count / 2]
}

// Full rows only; partial clipped boundary rows are excluded from the denominator.
func ink(_ data: Data, _ frame: [String: Any], _ rect: CGRect) -> Int {
    let stride = frame["stride"] as! Int, width = frame["width"] as! Int, height = frame["height"] as! Int
    let x0 = max(0, Int(rect.minX)), x1 = min(width, Int(rect.maxX))
    let y0 = max(0, Int(rect.minY)), y1 = min(height, Int(rect.maxY))
    if x0 >= x1 || y0 >= y1 { return 0 }
    return data.withUnsafeBytes { bytes in
        let base = bytes.bindMemory(to: UInt8.self); var count = 0
        for y in y0..<y1 {
            for x in x0..<x1 {
                let i = y * stride + x * 4
                if Int(base[i]) + Int(base[i+1]) + Int(base[i+2]) > 360 { count += 1 }
            }
        }
        return count
    }
}

struct Anchor { let id: Int; let center: Double }

func matchesSource(_ id: Int, _ center: Double, _ source: [String:Any], _ panel: String, _ shift: Double, _ pitch: Double, _ viewportY: Double, _ scale: Double) -> Bool {
    guard let offset = source["offset_y"] as? Double else { return false }
    if panel == "editor", let rows = source["rows"] as? [[String:Any]] {
        return rows.contains { row in
            guard row["id"] as? Int == id, let y = row["y"] as? Double else { return false }
            return abs(center-((y-viewportY)*scale+shift)) < pitch*0.45
        }
    }
    return Int(round((center-shift-offset*scale)/pitch)) + (panel == "grid" ? 1 : 0) == id
}

@main struct Capture {
    @MainActor static func main() async throws {
        let args = CommandLine.arguments
        if args[1] == "capture" {
            _ = NSApplication.shared
            _ = CGMainDisplayID()
            let pid = Int32(args[2])!, folder = URL(fileURLWithPath: args[3]), stop = args[4]
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
            var window: SCWindow?
            for _ in 0..<50 {
                let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
                window = content.windows.first { $0.owningApplication?.processID == pid && $0.frame.width > 600 }
                if window != nil { break }
                try await Task.sleep(for: .milliseconds(100))
            }
            guard let window else { throw NSError(domain: "F0 no native window", code: 1) }
            let configuration = SCStreamConfiguration()
            let viewport = try JSONSerialization.jsonObject(with: Data(contentsOf: folder.appendingPathComponent("viewport.json"))) as! [String:Any]
            let v = viewport.mapValues { ($0 as? NSNumber)?.doubleValue ?? 0 }
            configuration.sourceRect = CGRect(x:v["x"]!,y:v["y"]!,width:v["width"]!,height:v["height"]!)
            configuration.width = Int(v["width"]! * 2)
            configuration.height = Int(v["height"]! * 2)
            configuration.pixelFormat = kCVPixelFormatType_32BGRA
            configuration.minimumFrameInterval = CMTime(value: 1, timescale: 120)
            configuration.queueDepth = 8
            configuration.showsCursor = false
            configuration.ignoreShadowsSingleWindow = true
            let writer = try Writer(folder)
            let stream = SCStream(filter: SCContentFilter(desktopIndependentWindow: window), configuration: configuration, delegate: nil)
            try stream.addStreamOutput(writer, type: .screen, sampleHandlerQueue: DispatchQueue(label: "f0.frames"))
            try await stream.startCapture()
            for _ in 0..<300 {
                if FileManager.default.fileExists(atPath: stop) { break }
                try await Task.sleep(for: .milliseconds(100))
            }
            try await stream.stopCapture()
            try writer.raw.close(); try writer.meta.close()
            print("captured=\(writer.index)")
            return
        }
        if args[1] == "self-test" {
            let frame: [String: Any] = ["width":100,"height":60,"stride":400]
            let dark = Data(repeating: 32, count: 24000)
            precondition(ink(dark, frame, CGRect(x:0,y:0,width:100,height:60)) == 0)
            var glyph = dark
            for y in 10..<25 { for x in 10..<20 { for c in 0..<3 { glyph[y*400+x*4+c] = 230 } } }
            precondition(ink(glyph, frame, CGRect(x:0,y:0,width:100,height:30)) == 150)
            precondition(ink(glyph, frame, CGRect(x:0,y:30,width:100,height:30)) == 0)
            let ids = [10,11,999,13], bases = ids.enumerated().map { $0.element - $0.offset }
            let base = Dictionary(grouping:bases,by:{$0}).max{$0.value.count < $1.value.count}!.key
            precondition(ids.enumerated().filter{$0.element != base + $0.offset}.count == 1)
            precondition(matchesSource(7,440,["offset_y":0.0],"collection",20,60,0,2))
            precondition(!matchesSource(9999,440,["offset_y":0.0],"collection",20,60,0,2))
            precondition(matchesSource(17,440,["offset_y": -300.0],"collection",20,60,0,2))
            precondition(matchesSource(30,110,["offset_y": -30.0,"rows":[["id":30,"y":55.0]]],"editor",20,60,10,2))
            print("pixel classifier self-test: blank, painted, stale = PASS")
            return
        }
        let folder = URL(fileURLWithPath: args[2]), panel = args[3]
        let setup = try JSONSerialization.jsonObject(with: Data(contentsOf: folder.appendingPathComponent("setup.json"))) as! [String: Any]
        let viewport = setup["viewport"] as! [String: Double]
        let start = setup["start"] as! Double, end = setup["end"] as! Double
        let scale = viewport["scale"]!
        let frames = try String(contentsOf: folder.appendingPathComponent("frames.jsonl"), encoding: .utf8).split(separator: "\n").map { try JSONSerialization.jsonObject(with: Data($0.utf8)) as! [String: Any] }
        let raw = try Data(contentsOf: folder.appendingPathComponent("frames.bgra"), options: .mappedIfSafe)
        let source = (try? String(contentsOf: folder.appendingPathComponent("source.rs"), encoding: .utf8).components(separatedBy: "\n")) ?? []
        let output = folder.appendingPathComponent("rows.jsonl")
        FileManager.default.createFile(atPath: output.path, contents: nil)
        let out = try FileHandle(forWritingTo: output)
        var blank = 0, stale = 0, rows = 0, measured = 0, noAnchors = 0
        var lastPhase = 0.0, pitch = panel == "editor" ? 18.0 * scale : 30.0 * scale
        var dataTop = 0.0, lastBase = 0, gutterRight = 80.0
        var minimumID = Int.max, maximumID = Int.min
        var lastHash: UInt64?, changedAt = start, stalledRows = 0, stalledFrames = 0
        let inputs = setup["inputs"] as? [[Double]] ?? []
        let oracles = setup["oracles"] as? [[String:Any]] ?? []
        var textShift: Double?, missingOracleFrames = 0
        for frame in frames {
            let epoch = frame["epoch_ms"] as! Double
            let offset = frame["offset"] as! Int, count = frame["size"] as! Int
            let pixels = raw.subdata(in: offset..<offset+count)
            let width = frame["width"] as! Int, height = frame["height"] as! Int
            let dx = (Double(width) - viewport["window_width"]! * scale) / 2
            let dy = Double(height) - viewport["window_height"]! * scale - dx
            let roi = (setup["cropped"] as? Bool == true) ? CGRect(x:0,y:0,width:width,height:height) : CGRect(x: viewport["x"]! * scale + dx, y: viewport["y"]! * scale + dy, width: viewport["width"]! * scale, height: viewport["height"]! * scale)
            guard let image = makeImage(pixels, frame), let crop = image.cropping(to: roi) else { continue }
            if measured == 0 && epoch >= start {
                let png = CGImageDestinationCreateWithURL(folder.appendingPathComponent("sample.png") as CFURL, UTType.png.identifier as CFString, 1, nil)!
                CGImageDestinationAddImage(png, crop, nil); CGImageDestinationFinalize(png)
            }
            let request = VNRecognizeTextRequest()
            request.recognitionLevel = .fast; request.usesLanguageCorrection = false
            request.recognitionLanguages = ["en-US"]; request.minimumTextHeight = 0.007
            request.customWords = ["F0", "row", "Product"]
            request.regionOfInterest = CGRect(x:0,y:0,width:min((panel == "editor" ? 100.0 : 320.0)/Double(crop.width),1),height:1)
            try VNImageRequestHandler(cgImage: crop).perform([request])
            let observations = request.results ?? []
            var anchors: [Anchor] = []
            var texts: [[String: Any]] = []
            for observation in observations {
                guard let text = observation.topCandidates(1).first else { continue }
                let box = observation.boundingBox
                let center = (1 - box.midY) * Double(crop.height)
                texts.append(["text":text.string,"confidence":text.confidence,"x":box.minX * Double(crop.width),"center":center])
                let id: Int?
                if panel == "editor" {
                    id = box.maxX * Double(crop.width) < 140 ? number(text.string, "^\\s*([0-9]+)\\s*$") : nil
                } else if panel == "grid" { id = number(text.string, "^Product\\s+([0-9]+)$") }
                else { id = number(text.string, "^[Ff][O0]\\s*row\\s*([0-9]+)$") }
                if let id {
                    anchors.append(Anchor(id:id, center:center))
                    if panel == "editor" { gutterRight = max(gutterRight, box.maxX*Double(crop.width)) }
                }
            }
            anchors.sort { $0.center < $1.center }
            if true {
                let gaps = zip(anchors, anchors.dropFirst()).filter { $1.id == $0.id+1 }.map { $1.center - $0.center }.filter { $0 > 5 }
                if !gaps.isEmpty { pitch = median(gaps) }
            }
            if panel == "grid" && epoch < start, let first = anchors.first { dataTop = max(0, first.center - pitch/2) }
            if let anchor = anchors.first { lastPhase = anchor.center.truncatingRemainder(dividingBy: pitch) }
            let nearby = oracles.filter { abs(($0["epoch_ms"] as! Double) - epoch) <= 25 }.compactMap { $0["source"] as? [String:Any] }
            if textShift == nil, epoch < start, let source = nearby.first, let offset = source["offset_y"] as? Double, let first = anchors.first {
                let oneBased = panel == "editor" || panel == "grid" ? 1 : 0
                if panel == "editor", let expectedRows = source["rows"] as? [[String:Any]], let row = expectedRows.first(where: { $0["id"] as? Int == first.id }), let y = row["y"] as? Double {
                    textShift = first.center - (y-viewport["y"]!)*scale
                } else if panel != "editor" { textShift = first.center - Double(first.id-oneBased)*pitch - offset*scale }
            }
            if epoch < start || epoch > end { continue }
            if nearby.isEmpty { missingOracleFrames += 1 }
            var frameBlank = 0, frameStale = 0, frameRows = 0
            if anchors.isEmpty { noAnchors += 1 }
            let bases = anchors.map { $0.id - Int(round(($0.center - lastPhase)/pitch)) }
            let majority = Dictionary(grouping: bases, by: { $0 }).max { $0.value.count < $1.value.count }?.key
            if let majority { lastBase = majority }
            let expectedBase = majority ?? lastBase
            let centers: [(Double,Int)]
            if panel == "editor", let shift = textShift, let sample = nearby.min(by: { abs(($0["offset_y"] as? Double ?? 0)) < abs(($1["offset_y"] as? Double ?? 0)) }), let expectedRows = sample["rows"] as? [[String:Any]] {
                centers = expectedRows.compactMap { row in guard let id = row["id"] as? Int, let y = row["y"] as? Double else { return nil }; return ((y-viewport["y"]!)*scale+shift,id) }
            } else if panel == "editor", !anchors.isEmpty { centers = anchors.map { ($0.center,$0.id) } }
            else {
                centers = stride(from:lastPhase,to:Double(crop.height),by:pitch).map { ($0,expectedBase+Int(round(($0-lastPhase)/pitch))) }
            }
            for (center,expected) in centers {
                if center - pitch*0.3 >= dataTop && center + pitch*0.3 <= Double(crop.height) {
                    let inData = panel == "editor" ? expected > 0 && expected <= source.count : panel == "grid" ? expected >= 1 && expected <= 1000 : expected >= 0 && expected < 5000
                    let isExpectedText = inData && (panel != "editor" || !source[expected-1].trimmingCharacters(in: .whitespaces).isEmpty)
                    if isExpectedText {
                        frameRows += 1
                        let left = panel == "editor" ? gutterRight + 8 : 12.0
                        let span = panel == "editor" ? Double(crop.width)-left-8 : min(480.0, Double(crop.width)-left-8)
                        let region = CGRect(x: roi.minX+left, y: roi.minY+center-pitch*0.3, width:span, height:pitch*0.6)
                        if ink(pixels, frame, region) < 4 { frameBlank += 1 }
                        if let anchor = anchors.first(where: { abs($0.center-center) < pitch*0.3 }), let shift = textShift, !nearby.isEmpty {
                            let oneBased = panel == "editor" || panel == "grid" ? 1 : 0
                            let matches = nearby.contains { source in
                                matchesSource(anchor.id, anchor.center, source, panel, shift, pitch, viewport["y"]!, scale)
                            }
                            if !matches { frameStale += 1 }
                        } else if let anchor = anchors.first(where: { abs($0.center-center) < pitch*0.3 }), oracles.isEmpty, majority != nil, anchor.id != expected { frameStale += 1 }
                    }
                }
            }
            // A held image during sustained input is reported separately from wrong row IDs.
            let fingerprint: UInt64 = pixels.withUnsafeBytes { bytes in
                let base = bytes.bindMemory(to: UInt8.self); var hash: UInt64 = 14695981039346656037
                for y in stride(from:max(0,Int(roi.minY)),to:min(height,Int(roi.maxY)),by:2) {
                    for x in stride(from:max(0,Int(roi.minX+12)),to:min(width,Int(roi.minX+min(480,roi.width-16))),by:2) {
                        let i = y*(frame["stride"] as! Int)+x*4
                        hash = (hash ^ UInt64(base[i])) &* 1099511628211
                        hash = (hash ^ UInt64(base[i+1])) &* 1099511628211
                        hash = (hash ^ UInt64(base[i+2])) &* 1099511628211
                    }
                }
                return hash
            }
            if lastHash != fingerprint { changedAt = epoch; lastHash = fingerprint }
            let recent = inputs.filter { $0[0] > changedAt && $0[0] <= epoch && abs($0[1]) > 1 }
            let direction = recent.last?.last ?? 0
            let firstID = anchors.first?.id ?? expectedBase, lastID = anchors.last?.id ?? expectedBase
            let maximum = panel == "editor" ? source.count : panel == "grid" ? 1000 : 4999
            let atLimit = (direction > 0 && firstID <= 1) || (direction < 0 && lastID >= maximum-1)
            let stalled = epoch-changedAt > 33.4 && recent.count >= 3 && !atLimit
            if stalled { stalledFrames += 1; stalledRows += frameRows }
            if let first = anchors.first { minimumID = min(minimumID, first.id) }
            if let last = anchors.last { maximumID = max(maximumID,last.id) }
            measured += 1; blank += frameBlank; stale += frameStale; rows += frameRows
            let result: [String: Any] = ["frame":frame["frame"]!,"epoch_ms":epoch,"rows":frameRows,"blank":frameBlank,"stale":frameStale,"anchors":anchors.count,"pitch":pitch,"stalled":stalled,"texts":texts]
            try out.write(contentsOf: JSONSerialization.data(withJSONObject: result)+Data([10]))
        }
        try out.close()
        print(String(data: try JSONSerialization.data(withJSONObject:["frames":measured,"row_observations":rows,"blank_rows":blank,"stale_rows":stale,"frames_without_ocr_anchors":noAnchors,"stalled_frames":stalledFrames,"stalled_row_observations":stalledRows,"minimum_id":minimumID,"maximum_id":maximumID,"frames_without_oracle":missingOracleFrames], options:.sortedKeys),encoding:.utf8)!)
    }
}
