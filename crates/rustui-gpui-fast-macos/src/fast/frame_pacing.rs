// RustUI G5-2: runtime-selected NSView display link, main run loop only.
use objc2::{
    DefinedClass, Encode, Encoding, MainThreadMarker, MainThreadOnly, Message, define_class,
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject},
    sel,
};
use objc2_app_kit::NSView;
use objc2_foundation::{
    NSObject, NSObjectProtocol, NSProcessInfo, NSRunLoop, NSRunLoopCommonModes,
};
#[repr(C)]
#[derive(Clone, Copy)]
struct RateRange {
    minimum: f32,
    maximum: f32,
    preferred: f32,
}
unsafe impl Encode for RateRange {
    const ENCODING: Encoding = Encoding::Struct(
        "CAFrameRateRange",
        &[Encoding::Float, Encoding::Float, Encoding::Float],
    );
}

fn set_paused(link: &AnyObject, value: bool) {
    unsafe {
        let _: () = msg_send![link,setPaused:value];
    }
}
fn invalidate(link: &AnyObject) {
    unsafe {
        let _: () = msg_send![link, invalidate];
    }
}
use gpui::{FrameRequestSource, PlatformFrameSignal, WindowId};
use std::{cell::Cell, ffi::c_void, rc::Rc, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Choice {
    Legacy,
    Metal,
    View,
}
fn select(
    major: i64,
    view_available: bool,
    metal_available: bool,
    view: bool,
    legacy: bool,
) -> Choice {
    if legacy || major < 14 {
        Choice::Legacy
    } else if view && view_available {
        Choice::View
    } else if metal_available {
        Choice::Metal
    } else {
        Choice::Legacy
    }
}
pub(crate) fn choose(view: &NSView, metal_available: bool, window: WindowId) -> Choice {
    let major = NSProcessInfo::processInfo()
        .operatingSystemVersion()
        .majorVersion as i64;
    let available = major >= 14
        && AnyClass::get(c"CADisplayLink").is_some()
        && view.respondsToSelector(sel!(displayLinkWithTarget:selector:));
    let choice = select(
        major,
        available,
        metal_available,
        gpui::fast_view_link_requested(),
        gpui::fast_force_legacy_link(),
    );
    gpui::fast_latency_config(
        window,
        "g5_frame_source",
        [
            choice as u8 as f64,
            major as f64,
            available as u8 as f64,
            gpui::fast_force_legacy_link() as u8 as f64,
        ],
    );
    choice
}
pub(crate) struct NativeTick(std::sync::atomic::AtomicU64);
impl Default for NativeTick {
    fn default() -> Self {
        Self(std::sync::atomic::AtomicU64::new(0))
    }
}
impl NativeTick {
    pub(crate) fn record(&self) {
        if gpui::fast_latency_enabled() {
            let _ = self.0.compare_exchange(
                0,
                gpui::fast_latency_tick(),
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            );
        }
    }
    pub(crate) fn take(&self) -> u64 {
        self.0.swap(0, std::sync::atomic::Ordering::AcqRel)
    }
}
struct Inner {
    tick: NativeTick,
    window: WindowId,
    data: *mut c_void,
    callback: extern "C" fn(*mut c_void),
    used: Cell<bool>,
    woken: Cell<bool>,
    active: Cell<Option<bool>>,
    signal: Arc<PlatformFrameSignal>,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind=MainThreadOnly]
    #[ivars=Rc<Inner>]
    struct RustUIFastViewTarget;
    unsafe impl NSObjectProtocol for RustUIFastViewTarget {}
    impl RustUIFastViewTarget {
        #[unsafe(method(tick:))]
        fn tick(&self,link:&AnyObject) {
            let target=self.retain();let inner=target.ivars();inner.tick.record();inner.used.set(false);inner.woken.set(false);
            let active=gpui::fast_frame_input_active(inner.window);set_rate(link,inner,active);
            if let Some(at)=PlatformFrameSignal::capture(scheduler::Instant::now){inner.signal.record(at,FrameRequestSource::NativeCallback);}
            (inner.callback)(inner.data);
            if !inner.used.get() && !inner.woken.get() && !active {set_rate(link,inner,false);set_paused(link,true);}
        }
    }
);
fn set_rate(link: &AnyObject, inner: &Inner, active: bool) {
    if inner.active.replace(Some(active)) == Some(active) {
        return;
    }
    let (minimum, maximum, preferred) = if active {
        (80., 120., 120.)
    } else {
        (0., 0., 0.)
    };
    unsafe {
        let _: () = msg_send![link,setPreferredFrameRateRange:RateRange{minimum,maximum,preferred}];
    }
    let actual: RateRange = unsafe { msg_send![link, preferredFrameRateRange] };
    gpui::fast_latency_config(
        inner.window,
        "g5_view_rate",
        [
            actual.minimum as f64,
            actual.maximum as f64,
            actual.preferred as f64,
            0.,
        ],
    );
}
pub(crate) struct ViewFrames {
    link: Retained<AnyObject>,
    _target: Retained<RustUIFastViewTarget>,
    inner: Rc<Inner>,
}
impl ViewFrames {
    pub(crate) fn new(
        view: &NSView,
        window: WindowId,
        data: *mut c_void,
        callback: extern "C" fn(*mut c_void),
        main: MainThreadMarker,
    ) -> Self {
        let inner = Rc::new(Inner {
            tick: Default::default(),
            window,
            data,
            callback,
            used: Cell::new(false),
            woken: Cell::new(false),
            active: Cell::new(None),
            signal: Arc::new(PlatformFrameSignal::new()),
        });
        let target = RustUIFastViewTarget::alloc(main).set_ivars(inner.clone());
        let target: Retained<RustUIFastViewTarget> = unsafe { msg_send![super(target), init] };
        // The caller selects this path only after the OS and selector checks.
        let link: Retained<AnyObject> =
            unsafe { msg_send![view,displayLinkWithTarget:&*target,selector:sel!(tick:)] };
        set_paused(&link, true);
        unsafe {
            let _: () = msg_send![&*link,addToRunLoop:&*NSRunLoop::mainRunLoop(),forMode:NSRunLoopCommonModes];
        }
        Self {
            link,
            _target: target,
            inner,
        }
    }
    pub(crate) fn take_native_tick(&self) -> u64 {
        self.inner.tick.take()
    }
    pub(crate) fn wake(&self) {
        self.inner.woken.set(true);
        set_paused(&self.link, false);
    }
    pub(crate) fn close(&self) {
        set_rate(&self.link, &self.inner, false);
        invalidate(&self.link);
    }
    pub(crate) fn pause(&self) {
        set_rate(&self.link, &self.inner, false);
        set_paused(&self.link, true);
    }
    pub(crate) fn used(&self) {
        self.inner.used.set(true);
    }
    pub(crate) fn take_signal(&self) -> Option<(scheduler::Instant, FrameRequestSource)> {
        self.inner.signal.take()
    }
}
impl Drop for ViewFrames {
    fn drop(&mut self) {
        invalidate(&self.link);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_selection_is_runtime_and_conservative() {
        assert_eq!(select(13, true, true, true, false), Choice::Legacy);
        assert_eq!(select(12, false, false, false, false), Choice::Legacy);
        assert_eq!(select(14, true, true, false, false), Choice::Metal);
        assert_eq!(select(14, true, true, true, false), Choice::View);
        assert_eq!(select(27, false, true, true, false), Choice::Metal);
        assert_eq!(select(27, true, true, true, true), Choice::Legacy);
        assert_eq!(select(14, false, false, true, false), Choice::Legacy);
    }
}

pub(crate) struct LegacyFrames {
    source: std::cell::RefCell<crate::display_link::WindowFrameSource>,
    running: Cell<bool>,
    pending: Cell<bool>,
}
impl LegacyFrames {
    pub(crate) fn new(data: *mut c_void, callback: extern "C" fn(*mut c_void)) -> Self {
        Self {
            source: std::cell::RefCell::new(crate::display_link::WindowFrameSource::new(
                data, callback,
            )),
            running: Cell::new(false),
            pending: Cell::new(false),
        }
    }
    pub(crate) fn start(&self, id: core_graphics::display::CGDirectDisplayID) {
        use gpui_util::ResultExt;
        self.running
            .set(self.source.borrow_mut().start(id).log_err().is_some());
    }
    pub(crate) fn take_native_tick(&self) -> u64 {
        self.source.borrow().take_native_tick()
    }
    pub(crate) fn running(&self) -> bool {
        self.running.get()
    }
    pub(crate) fn stop(&self) {
        self.running.set(false);
        self.source.borrow_mut().stop();
    }
    pub(crate) fn take_signal(&self) -> Option<(scheduler::Instant, FrameRequestSource)> {
        self.source.borrow().take_signal()
    }
    pub(crate) fn waker(
        this: &Rc<Self>,
        window: WindowId,
        executor: gpui::ForegroundExecutor,
        recreate: impl Fn() + 'static,
    ) -> Rc<dyn Fn()> {
        let weak = Rc::downgrade(this);
        let recreate = Rc::new(recreate);
        Rc::new(move || {
            if gpui::fast_immediate_demand(window) {
                return;
            }
            let Some(source) = weak.upgrade() else { return };
            if source.running.get() || source.pending.replace(true) {
                return;
            }
            let (weak, recreate) = (weak.clone(), recreate.clone());
            executor
                .spawn(async move {
                    if let Some(source) = weak.upgrade() {
                        if !source.running.get() {
                            recreate();
                        }
                        source.pending.set(false);
                    }
                })
                .detach();
        })
    }
}
