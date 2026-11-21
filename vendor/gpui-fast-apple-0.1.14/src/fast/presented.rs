// Added by RustUI for PX-2. No draw/present decision is changed.
use block2::RcBlock;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLDrawable;
use objc2_quartz_core::CAMetalDrawable;
use std::ptr::NonNull;
// CABase is not enabled in the pinned bindings; this is the SDK's exact
// CFTimeInterval signature. QuartzCore is already linked by the renderer.
#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C-unwind" {
    fn CACurrentMediaTime() -> f64;
}
pub(crate) fn observe(drawable: &ProtocolObject<dyn CAMetalDrawable>) {
    let Some(tag) = gpui::fast_present_tag() else {
        return;
    };
    let block = RcBlock::new(move |drawable: NonNull<ProtocolObject<dyn MTLDrawable>>| {
        // SAFETY: Metal supplies a live drawable; the SDK's result is host seconds.
        // Dispatch directly because the generated method is CF-feature gated.
        let presented: f64 = unsafe { objc2::msg_send![drawable.as_ref(), presentedTime] };
        gpui::fast_frame_presented(tag, presented, unsafe { CACurrentMediaTime() });
    });
    // SAFETY: Metal copies the valid block until the drawable is displayed.
    unsafe {
        drawable.addPresentedHandler(RcBlock::as_ptr(&block));
    }
}
