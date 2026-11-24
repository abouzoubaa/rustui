// Shared G5 capture path: identical filter, configuration, decoder and clock.
import AppKit
import ScreenCaptureKit
import CoreMedia
import CoreVideo
import QuartzCore
import Darwin
struct VersionTransitions {
 var last:Int?
 mutating func observe(_ version:Int)->Bool {
  guard last != version else{return false}
  last=version;return true
 }
}
final class VersionCapture:NSObject,SCStreamOutput {
 let queue=DispatchQueue(label:"g5.shared.pixels")
 var stream:SCStream!;var transitions=VersionTransitions();let emit:([String:Any])->Void
 let cadence: ((UInt64, Int) -> Void)?
 init(cadence: ((UInt64, Int) -> Void)? = nil, emit:@escaping ([String:Any])->Void) {
  self.cadence = cadence
  self.emit = emit
 }
 @MainActor func start(window:NSWindow) async throws {
  let scale=window.backingScaleFactor
  let width=Int(window.frame.width*scale), height=Int(window.frame.height*scale)
  let id=CGWindowID(window.windowNumber)
  if let directory=ProcessInfo.processInfo.environment["RUSTUI_L1_CAPTURE_DIR"] {
   let root=URL(fileURLWithPath:directory)
   try "\(id) \(width) \(height)".write(to:root.appendingPathComponent("window"),atomically:true,encoding:.utf8)
   let deadline=Date().addingTimeInterval(10)
   while !FileManager.default.fileExists(atPath:root.appendingPathComponent("ready").path) {
    guard Date()<deadline else{throw NSError(domain:"G5ObserverTimeout",code:1)}
    try await Task.sleep(nanoseconds:10_000_000)
   }
   return
  }
  try await start(id:id,width:width,height:height)
 }
 @MainActor func start(id:CGWindowID,width:Int,height:Int) async throws {
  guard CGPreflightScreenCaptureAccess() else {throw NSError(domain:"G5ScreenAccess",code:1)}
  // WindowServer can publish a newly ordered window after AppKit returns.
  // Wait only during setup, before either app's normal warmup starts.
  let deadline=Date().addingTimeInterval(5);var selected:SCWindow?
  while selected==nil && Date()<deadline {
   let content=try await SCShareableContent.excludingDesktopWindows(false,onScreenWindowsOnly:true)
   selected=content.windows.first(where:{$0.windowID==id})
   if selected==nil {try await Task.sleep(nanoseconds:10_000_000)}
  }
  guard let target=selected else {throw NSError(domain:"G5WindowMissing",code:1)}
  let config=SCStreamConfiguration()
  config.width=width;config.height=height
  config.minimumFrameInterval=CMTime(value:1,timescale:120);config.pixelFormat=kCVPixelFormatType_32BGRA;config.queueDepth=3;config.showsCursor=false;config.colorSpaceName=CGColorSpace.sRGB
  stream=SCStream(filter:SCContentFilter(desktopIndependentWindow:target),configuration:config,delegate:nil)
  try stream.addStreamOutput(self,type:.screen,sampleHandlerQueue:queue)
  try await stream.startCapture()
 }
 @MainActor func stop() async {if let stream {try? await stream.stopCapture()};queue.sync{}}
 func stream(_ stream:SCStream,didOutputSampleBuffer buffer:CMSampleBuffer,of type:SCStreamOutputType) {
  guard type == .screen,let attachments=CMSampleBufferGetSampleAttachmentsArray(buffer,createIfNecessary:false) as? [[SCStreamFrameInfo:Any]],let info=attachments.first,
   let status=info[.status] as? Int,let display=info[.displayTime] as? UInt64,display>0 else{return}
  cadence?(display, status)
  guard status == SCFrameStatus.complete.rawValue, let image = buffer.imageBuffer else { return }
  CVPixelBufferLockBaseAddress(image,.readOnly);defer{CVPixelBufferUnlockBaseAddress(image,.readOnly)}
  guard let ptr=CVPixelBufferGetBaseAddress(image)?.assumingMemoryBound(to:UInt8.self) else{return}
  let width=CVPixelBufferGetWidth(image),height=CVPixelBufferGetHeight(image),stride=CVPixelBufferGetBytesPerRow(image)
  func red(_ x:Int,_ y:Int)->Bool {let i=y*stride+x*4;return ptr[i+2]>220 && ptr[i+1]<35 && ptr[i]<35}
  for y in Swift.stride(from:8,to:height-8,by:16) {for x in Swift.stride(from:4,to:width-4,by:4) {
   guard red(x,y) else{continue};var left=x;var right=x
   while left>0 && red(left-1,y){left-=1};while right+1<width && red(right+1,y){right+=1}
   let unit=right-left+1;guard unit>=3,unit<=20,left+26*unit<=width else{continue}
   var bits=[Int]();var valid=true
   for column in 1...24 {let i=y*stride+(left+column*unit+unit/2)*4;let r=ptr[i+2],g=ptr[i+1],b=ptr[i]
    if r>220 && g>220 && b>220 {bits.append(1)}else if r<35 && g<35 && b<35 {bits.append(0)}else{valid=false;break}
   }
   guard valid,bits.count==24,bits.prefix(8).reduce(0,{$0*2+$1})==0xa5,red(left+25*unit+unit/2,y) else{continue}
   let v=bits.dropFirst(8).reduce(0,{$0*2+$1});guard v>=0,v<=10000 else{continue}
   if transitions.observe(v) {emit(["event":"displayed_pixels","version":v,"display_mach":display,"callback_mach":mach_absolute_time(),"pixel_x":left,"pixel_y":y])};return
  }}
 }
}
private var capture:VersionCapture?;private var captureState:Int32=0
private let pixelsLock=NSLock();private var pixelRecords=[[String:Any]]()
@_cdecl("g5_pixels_start") public func g5PixelsStart(){
 guard ProcessInfo.processInfo.environment["RUSTUI_G5_PIXEL_CAPTURE"]=="1" else{return}
 captureState=0
 Task {@MainActor in
  let deadline=Date().addingTimeInterval(5);var selected:NSWindow?
  while selected==nil && Date()<deadline {
   selected=NSApp.keyWindow ?? NSApp.mainWindow ?? NSApp.windows.first(where:{$0.isVisible && $0.contentView != nil})
   if selected==nil {try? await Task.sleep(nanoseconds:10_000_000)}
  }
  guard let window=selected else {fputs("G5 capture has no visible reference window\n",stderr);captureState = -1;return}
  capture=VersionCapture {r in pixelsLock.withLock{pixelRecords.append(r)}}
  do{try await capture!.start(window:window);captureState=1}catch{fputs("G5 pixel capture failed: \(error)\n",stderr);captureState = -1}
 }

}
@_cdecl("g5_pixels_state") public func g5PixelsState()->Int32{captureState}
@_cdecl("g5_pixels_stop") public func g5PixelsStop(){
 Task {@MainActor in await capture?.stop();let rows=pixelsLock.withLock{pixelRecords};var output="";for row in rows {let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]);output += "G5_PIXELS "+String(data:data,encoding:.utf8)!+"\n"};fputs(output,stderr);capture=nil;captureState=2}
}
