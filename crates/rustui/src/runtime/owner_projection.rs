//! Conservative notification proof for already-painted native editor bindings.
//! Unsupported descriptions keep the normal GPUI notification path.
use super::component_host::RetainedEntities;
use crate::view::ComponentUpdater;
use gpui::{App, WindowId};
use std::{any::Any, cell::Cell, rc::Rc};
trait Value {
    fn as_any(&self) -> &dyn Any;
    fn equal(&self, other: &dyn Value) -> bool;
}
impl<T: PartialEq + 'static> Value for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn equal(&self, other: &dyn Value) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .is_some_and(|v| self == v)
    }
}
type Check = Rc<dyn Fn(&RetainedEntities, &App) -> bool>;
type Commit = Rc<dyn Fn(&RetainedEntities, &mut App, ComponentUpdater, WindowId)>;
/// Internal exact descriptor comparison; never hashes pixels or runs layout.
#[doc(hidden)]
pub struct OwnerProjection {
    values: Vec<Box<dyn Value>>,
    checks: Vec<Check>,
    commits: Vec<Commit>,
}
impl OwnerProjection {
    pub(crate) fn leaf<T: PartialEq + 'static>(props: T) -> Self {
        Self {
            values: vec![Box::new(props)],
            checks: Vec::new(),
            commits: Vec::new(),
        }
    }
    pub(crate) fn with_native(mut self, check: Check, commit: Commit) -> Self {
        self.checks.push(check);
        self.commits.push(commit);
        self
    }
    pub(crate) fn and(mut self, mut child: Self) -> Self {
        self.values.append(&mut child.values);
        self.checks.append(&mut child.checks);
        self.commits.append(&mut child.commits);
        self
    }
    pub(crate) fn same_shape(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self
                .values
                .iter()
                .zip(&other.values)
                .all(|(a, b)| a.equal(b.as_ref()))
    }
    pub(crate) fn already_painted(&self, retained: &RetainedEntities, cx: &App) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|f| f(retained, cx))
    }
    pub(crate) fn commit(
        &self,
        retained: &RetainedEntities,
        cx: &mut App,
        updater: ComponentUpdater,
        window: WindowId,
    ) {
        for f in &self.commits {
            f(retained, cx, updater.clone(), window);
        }
    }
}
thread_local! {static BINDING:Cell<u8>=const{Cell::new(0)};}
pub(crate) fn binding_effect<R>(native: bool, f: impl FnOnce() -> R) -> R {
    struct Restore(u8);
    impl Drop for Restore {
        fn drop(&mut self) {
            BINDING.with(|s| s.set(self.0));
        }
    }
    let old = BINDING.with(|s| s.replace(s.get() | 1 | if native { 2 } else { 0 }));
    let _restore = Restore(old);
    f()
}
pub(crate) fn binding_active() -> bool {
    BINDING.with(|s| s.get() & 1 != 0)
}
pub(crate) fn native_binding_active() -> bool {
    gpui::fast_draw_first_enabled() && BINDING.with(|s| s.get() & 2 != 0)
}
#[cfg(test)]
thread_local! {static SUPPRESSED:Cell<usize>=const{Cell::new(0)};}
pub(crate) fn record_suppressed() {
    #[cfg(test)]
    SUPPRESSED.with(|s| s.set(s.get() + 1));
}
#[cfg(test)]
pub(crate) fn suppressed() -> usize {
    SUPPRESSED.with(Cell::get)
}

// Prove that the actual component entered View's default body lowering.
// Inspecting body() is insufficient when into_element() is overridden.
#[derive(Clone, Copy)]
struct Lowering {
    ty: std::any::TypeId,
    address: *const (),
    default_body: bool,
}
thread_local! { static LOWERING: Cell<Option<Lowering>> = const { Cell::new(None) }; }
pub(crate) fn with_render_proof<V: 'static, R>(view: &V, f: impl FnOnce() -> R) -> (R, bool) {
    if !gpui::fast_draw_first_enabled() {
        return (f(), false);
    }
    struct Restore(Option<Lowering>);
    impl Drop for Restore {
        fn drop(&mut self) {
            LOWERING.with(|v| v.set(self.0));
        }
    }
    let previous = LOWERING.with(|v| {
        v.replace(Some(Lowering {
            ty: std::any::TypeId::of::<V>(),
            address: std::ptr::from_ref(view).cast(),
            default_body: false,
        }))
    });
    let _restore = Restore(previous);
    let result = f();
    let default_body = LOWERING.with(|v| v.get().is_some_and(|r| r.default_body));
    (result, default_body)
}
pub(crate) fn note_default_body<V: 'static>(view: &V) {
    if !gpui::fast_draw_first_enabled() || std::mem::size_of::<V>() == 0 {
        return;
    }
    LOWERING.with(|v| {
        if let Some(mut root) = v.get()
            && root.ty == std::any::TypeId::of::<V>()
            && root.address == std::ptr::from_ref(view).cast()
        {
            root.default_body = true;
            v.set(Some(root));
        }
    });
}
