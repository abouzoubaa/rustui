// RustUI G5-3d: retain GPUI's observer and cross-window notification semantics.
use crate::{App, EntityId, WindowId};
impl App {
    #[doc(hidden)]
    pub fn fast_owner_projection_allowed(&self, owner: EntityId, window: WindowId) -> bool {
        crate::fast_draw_first_enabled()
            && !self.observers.has_subscribers(&owner)
            && self
                .windows
                .get(window)
                .and_then(|w| w.as_ref())
                .is_some_and(|w| {
                    if w.root
                        .as_ref()
                        .is_some_and(|root| root.entity_id() == owner)
                    {
                        return true;
                    }
                    // A nested native host can be skipped only when GPUI's actual
                    // retained records prove that no other view read its state.
                    // Ancestor aggregate dependencies include children, so use the
                    // per-subtree own reads, never the aggregate dependency union.
                    let records = &w.rendered_frame.retained.records;
                    let seen = records
                        .iter()
                        .any(|r| crate::fast::splice::view_entity(&r.id) == Some(owner));
                    seen && records.iter().all(|r| {
                        !r.own_dependencies.entities.contains(&owner)
                            || crate::fast::splice::view_entity(&r.id) == Some(owner)
                    })
                })
            && self
                .tracked_entities
                .iter()
                .all(|(id, entities)| *id == window || !entities.contains(&owner))
    }
}
