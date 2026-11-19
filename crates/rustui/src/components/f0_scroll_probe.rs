//! Opt-in, read-only source-position oracle for the F0 native pixel recorder.
use crate::view::RenderContext;
use gpui::{App, Entity, EntityId, Global, WeakEntity, Window, WindowId};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Default)]
struct Probes(RefCell<HashSet<(WindowId, EntityId)>>);
impl Global for Probes {}
type Read<T> = Rc<dyn Fn(&T, &App) -> serde_json::Value>;

fn next<T: 'static>(
    target: WeakEntity<T>,
    read: Read<T>,
    key: (WindowId, EntityId),
    window: &Window,
) {
    window.on_next_frame(move |window, app| {
        let Some(entity) = target.upgrade() else {
            app.global::<Probes>().0.borrow_mut().remove(&key);
            return;
        };
        let value = read(entity.read(app), app);
        drop(entity);
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        eprintln!(
            "RUSTUI_ORACLE_LOG {}",
            serde_json::json!({"epoch_ms":epoch,"source":value})
        );
        next(target, read, key, window);
    });
}

pub(crate) fn watch<T: 'static>(
    context: &mut RenderContext<'_>,
    panel: &'static str,
    entity: &Entity<T>,
    read: impl Fn(&T, &App) -> serde_json::Value + 'static,
) {
    if context.app().background_executor().is_test()
        || std::env::var("RUSTUI_F0_PIXELS").as_deref() != Ok("1")
        || std::env::var("RUSTUI_F0_PANEL")
            .as_deref()
            .unwrap_or("collection")
            != panel
    {
        return;
    }
    let target = entity.downgrade();
    let id = entity.entity_id();
    context.update_entity(entity, move |_, window, cx| {
        if cx.try_global::<Probes>().is_none() {
            cx.set_global(Probes::default());
        }
        let key = (window.window_handle().window_id(), id);
        if cx.global::<Probes>().0.borrow_mut().insert(key) {
            next(target, Rc::new(read), key, window);
        }
    });
}
