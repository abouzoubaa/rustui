// G0 diagnostic only. Capture is separate from performance measurements.
import AppKit
import ScreenCaptureKit
import ImageIO
import UniformTypeIdentifiers

func pixels(_ image: CGImage) -> [UInt8] {
    var bytes=[UInt8](repeating:0,count:image.width*image.height*4)
    bytes.withUnsafeMutableBytes { buffer in
        let context=CGContext(data:buffer.baseAddress,width:image.width,height:image.height,bitsPerComponent:8,bytesPerRow:image.width*4,space:CGColorSpaceCreateDeviceRGB(),bitmapInfo:CGImageAlphaInfo.premultipliedLast.rawValue)!
        context.draw(image,in:CGRect(x:0,y:0,width:image.width,height:image.height))
    }
    return bytes
}
func startTime(_ log: URL) throws -> Double? {
    let text=try String(contentsOf:log,encoding:.utf8)
    for line in text.split(separator:"\n") where line.hasPrefix("RUSTUI_G0_LOG ") {
        let record=try JSONSerialization.jsonObject(with:Data(line.dropFirst(14).utf8)) as! [String:Any]
        if record["event"] as? String == "start" { return record["epoch_ms"] as? Double }
    }
    return nil
}
func pause(until epoch: Double) async throws {
    while Date().timeIntervalSince1970*1000 < epoch { try await Task.sleep(for:.milliseconds(10)) }
}
func save(_ image: CGImage,_ url: URL) throws {
    let writer=CGImageDestinationCreateWithURL(url as CFURL,UTType.png.identifier as CFString,1,nil)!
    CGImageDestinationAddImage(writer,image,nil)
    guard CGImageDestinationFinalize(writer) else { throw NSError(domain:"G0 PNG write",code:1) }
}
@main struct Main {
    @MainActor static func main() async throws {
        _ = NSApplication.shared
        _ = CGMainDisplayID()
        let args=CommandLine.arguments,pid=Int32(args[1])!,folder=URL(fileURLWithPath:args[2]),log=URL(fileURLWithPath:args[3])
        try FileManager.default.createDirectory(at:folder,withIntermediateDirectories:true)
        var start:Double?
        for _ in 0..<200 { start=try? startTime(log);if start != nil { break };try await Task.sleep(for:.milliseconds(50)) }
        guard let start else { throw NSError(domain:"G0 start marker missing",code:1) }
        let content=try await SCShareableContent.excludingDesktopWindows(false,onScreenWindowsOnly:true)
        guard let window=content.windows.first(where:{$0.owningApplication?.processID==pid && $0.frame.width>600}) else { throw NSError(domain:"G0 window missing",code:1) }
        let config=SCStreamConfiguration();config.width=Int(window.frame.width*2);config.height=Int(window.frame.height*2);config.showsCursor=false;config.ignoreShadowsSingleWindow=true
        let filter=SCContentFilter(desktopIndependentWindow:window)
        try await pause(until:start+800)
        let a=try await SCScreenshotManager.captureImage(contentFilter:filter,configuration:config)
        let aTime=Date().timeIntervalSince1970*1000
        try await pause(until:start+2800)
        let b=try await SCScreenshotManager.captureImage(contentFilter:filter,configuration:config)
        let bTime=Date().timeIntervalSince1970*1000
        guard aTime<start+1800 && bTime<start+3800 && a.width==b.width && a.height==b.height else { throw NSError(domain:"G0 capture missed stable row holds",code:1) }
        try save(a,folder.appendingPathComponent("row-2.png"));try save(b,folder.appendingPathComponent("row-8.png"))
        let av=pixels(a),bv=pixels(b),w=a.width,h=a.height,x0=Int(Double(w)*0.60),x1=Int(Double(w)*0.65)
        func stripe(_ bytes:[UInt8],_ y:Int)->[Double] {
            var sums=[Double](repeating:0,count:3)
            for x in x0..<x1 { for c in 0..<3 { sums[c] += Double(bytes[(y*w+x)*4+c]) } }
            return sums.map { $0/Double(x1-x0) }
        }
        func distance(_ a:[Double],_ b:[Double])->Double {
            var sum=0.0;for c in 0..<3 { sum += abs(a[c]-b[c]) };return sum
        }
        let ar=(0..<h).map{stripe(av,$0)},br=(0..<h).map{stripe(bv,$0)}
        var peak=0
        for y in 1..<h { if distance(ar[y],br[y])>distance(ar[peak],br[peak]) { peak=y } }
        let channel=(0..<3).max{abs(ar[peak][$0]-br[peak][$0])<abs(ar[peak][$1]-br[peak][$1])}!
        var bands:[(Int,Int,Int)]=[],begin=0,last=0
        for y in 0...h {
            let d=y<h ? ar[y][channel]-br[y][channel] : 0
            let sign=d>1 ? 1 : d < -1 ? -1 : 0
            if sign != last { if last != 0 { bands.append((begin,y,last)) };begin=y;last=sign }
        }
        guard let positive=bands.filter({$0.2==1}).max(by:{$0.1-$0.0<$1.1-$1.0}),let negative=bands.filter({$0.2 == -1}).max(by:{$0.1-$0.0<$1.1-$1.0}) else { throw NSError(domain:"G0 old/new row did not swap highlight pixels",code:1) }
        let scale=Double(w)/1200
        var proof:[String:Any]?
        for pair in [(positive,negative),(negative,positive)] { for flip in [false,true] {
            let oy=Double(pair.0.0+pair.0.1)/2,ny=Double(pair.1.0+pair.1.1)/2
            let topOld=flip ? Double(h)-oy : oy,topNew=flip ? Double(h)-ny : ny
            let offset=topOld-(32+15+2*30)*scale
            let oldHeight=pair.0.1-pair.0.0,newHeight=pair.1.1-pair.1.0
            let oldA=ar[Int(oy)],oldB=br[Int(oy)],newA=ar[Int(ny)],newB=br[Int(ny)]
            let changed=distance(oldA,oldB)
            let swapped=distance(oldA,newB)+distance(oldB,newA)
            if abs(topNew-topOld-180*scale)<4 && offset >= -4 && offset<=100 && oldHeight>=Int(25*scale) && newHeight>=Int(25*scale) && changed>5 && swapped<5 {
                proof=["verified":true,"old_row":2,"new_row":8,"old_center_px":oy,"new_center_px":ny,"old_before_rgb":oldA,"old_after_rgb":oldB,"new_before_rgb":newA,"new_after_rgb":newB,"width":w,"height":h,"scale":scale,"capture_a_epoch_ms":aTime,"capture_b_epoch_ms":bTime]
            }
        } }
        guard let proof else { throw NSError(domain:"G0 highlight pixels did not match both expected rows",code:1) }
        let json=try JSONSerialization.data(withJSONObject:proof,options:[.prettyPrinted,.sortedKeys]);try json.write(to:folder.appendingPathComponent("proof.json"));print(String(data:json,encoding:.utf8)!)
    }
}
