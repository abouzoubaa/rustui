//! Local compiled preview bookkeeping owned by an ordinary GPUI component.
use std::rc::Rc;

use crate::view::{AnyView, Keyed};

use super::registry::PreviewRegistry;
use super::{
    ActionRef, EventId, PreviewDocument, PreviewError, PreviewId, PropertyId, PropertyValue,
    ValidatedPreview, validate_preview,
};

/// Exact identity of a mounted descriptor revision. Tokens are local Rust values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewStamp {
    pub preview_id: PreviewId,
    pub generation: u64,
    pub revision: u64,
}

/// Immutable factory input. Factories cannot mutate the host or bypass validation.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSnapshot {
    pub stamp: PreviewStamp,
    pub preview: ValidatedPreview,
}

/// A signal contains only an event identifier and its exact mounted revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreviewSignal {
    pub stamp: PreviewStamp,
    pub event: EventId,
}

impl PreviewSnapshot {
    pub fn signal(&self, event: EventId) -> PreviewSignal {
        PreviewSignal {
            stamp: self.stamp.clone(),
            event,
        }
    }
}

/// Descriptor validation and compiled factory selection, without an event loop.
///
/// Store this value in a RustUI component. Its GPUI owner controls notification,
/// scheduling, bindings and release. `render` lowers through existing RustUI views.
/// Factories must use stable control IDs; property revisions keep retention scope,
/// while teardown/remount changes generation and discards native edit state.
pub struct PreviewHost<C> {
    registry: Rc<PreviewRegistry<C>>,
    mounted: Option<PreviewSnapshot>,
    generation: u64,
}

impl<C> PreviewHost<C> {
    pub fn new(registry: PreviewRegistry<C>) -> Self {
        Self {
            registry: Rc::new(registry),
            mounted: None,
            generation: 0,
        }
    }

    pub fn catalog(&self) -> &super::ComponentCatalog {
        self.registry.catalog()
    }

    pub fn snapshot(&self) -> Option<&PreviewSnapshot> {
        self.mounted.as_ref()
    }

    /// Validates first; a failed mount leaves the current instance untouched.
    pub fn mount(&mut self, document: PreviewDocument) -> Result<PreviewStamp, PreviewError> {
        let preview = validate_preview(self.catalog(), document)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| invalid("preview generation exhausted"))?;
        let stamp = PreviewStamp {
            preview_id: preview.document.preview_id.clone(),
            generation,
            revision: 0,
        };
        self.generation = generation;
        self.mounted = Some(PreviewSnapshot {
            stamp: stamp.clone(),
            preview,
        });
        Ok(stamp)
    }

    pub fn teardown(&mut self, stamp: &PreviewStamp) -> Result<(), PreviewError> {
        self.require_current(stamp)?;
        self.mounted = None;
        Ok(())
    }

    /// Applies one atomic typed inspector update batch to the exact revision.
    pub fn edit(
        &mut self,
        stamp: &PreviewStamp,
        updates: impl IntoIterator<Item = (PropertyId, PropertyValue)>,
    ) -> Result<PreviewStamp, PreviewError> {
        let current = self.require_current(stamp)?;
        let preview = current.preview.apply_batch(self.catalog(), updates)?;
        let revision = stamp
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("preview revision exhausted"))?;
        let next = PreviewStamp {
            revision,
            ..stamp.clone()
        };
        self.mounted = Some(PreviewSnapshot {
            stamp: next.clone(),
            preview,
        });
        Ok(next)
    }

    /// Feeds an ordinary typed owner binding value back through the same schema.
    /// This is a direct local call; values never travel inside event signals.
    pub fn binding_feedback(
        &mut self,
        stamp: &PreviewStamp,
        property: PropertyId,
        value: PropertyValue,
    ) -> Result<PreviewStamp, PreviewError> {
        self.edit(stamp, [(property, value)])
    }

    /// Resolves an exact current signal to its explicitly allowlisted target.
    /// The owner invokes its compiled handler after successful resolution.
    pub fn dispatch(&self, signal: &PreviewSignal) -> Result<ActionRef, PreviewError> {
        let current = self.require_current(&signal.stamp)?;
        current
            .preview
            .document
            .actions
            .get(&signal.event)
            .cloned()
            .ok_or_else(|| invalid("event has no registered action target"))
    }

    /// Builds the selected compiled view in its mounted retention scope.
    pub fn render(&self, context: &C) -> Result<Option<AnyView>, PreviewError> {
        let Some(snapshot) = &self.mounted else {
            return Ok(None);
        };
        let view = self.registry.render(snapshot, context)?;
        Ok(Some(AnyView::new(Keyed(
            format!(
                "compiled-preview-{}-{}",
                snapshot.stamp.preview_id, snapshot.stamp.generation
            ),
            view,
        ))))
    }

    /// Captures a read-only render description without cloning a live host.
    /// Callback stamps remain bound to this exact revision; the owner's live
    /// host validates them before any action or feedback is accepted.
    pub fn render_factory(&self) -> impl Fn(&C) -> Result<Option<AnyView>, PreviewError> + 'static
    where
        C: 'static,
    {
        let registry = self.registry.clone();
        let snapshot = self.mounted.clone();
        move |context| {
            let Some(snapshot) = &snapshot else {
                return Ok(None);
            };
            let view = registry.render(snapshot, context)?;
            Ok(Some(AnyView::new(Keyed(
                format!(
                    "compiled-preview-{}-{}",
                    snapshot.stamp.preview_id, snapshot.stamp.generation
                ),
                view,
            ))))
        }
    }

    fn require_current(&self, stamp: &PreviewStamp) -> Result<&PreviewSnapshot, PreviewError> {
        self.mounted
            .as_ref()
            .filter(|current| &current.stamp == stamp)
            .ok_or_else(|| invalid("stale or unmounted preview revision"))
    }
}

fn invalid(message: &str) -> PreviewError {
    PreviewError::Invalid(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{preview::*, view::EmptyView};
    use std::collections::BTreeMap;

    fn host() -> PreviewHost<()> {
        let mut registry = PreviewRegistry::new();
        registry
            .register(
                ComponentSchema {
                    id: ComponentId::new("text").unwrap(),
                    version: 1,
                    display_name: "Text".into(),
                    properties: BTreeMap::from([(
                        PropertyId::new("text").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::String,
                            default: Some(PropertyValue::String("hello".into())),
                            required: false,
                            editable: true,
                            minimum: None,
                            maximum: None,
                            min_length: None,
                            max_length: Some(100),
                            allowed_values: vec![],
                        },
                    )]),
                    events: BTreeMap::from([(
                        EventId::new("activate").unwrap(),
                        EventSchema {
                            display_name: "Activate".into(),
                            payload: BTreeMap::new(),
                        },
                    )]),
                },
                |_, _| AnyView::new(EmptyView),
            )
            .unwrap();
        registry
            .register_action(ActionRef::new("count").unwrap())
            .unwrap();
        PreviewHost::new(registry)
    }

    fn document() -> PreviewDocument {
        let mut document = PreviewDocument::new(
            PreviewId::new("specimen").unwrap(),
            ComponentId::new("text").unwrap(),
            1,
        );
        document.actions.insert(
            EventId::new("activate").unwrap(),
            ActionRef::new("count").unwrap(),
        );
        document
    }

    #[test]
    fn revisions_reject_stale_actions_edits_and_feedback() {
        let mut host = host();
        let initial = host.mount(document()).unwrap();
        let signal = host
            .snapshot()
            .unwrap()
            .signal(EventId::new("activate").unwrap());
        assert_eq!(host.dispatch(&signal).unwrap().as_str(), "count");
        let next = host
            .edit(
                &initial,
                [(
                    PropertyId::new("text").unwrap(),
                    PropertyValue::String("edited".into()),
                )],
            )
            .unwrap();
        assert_eq!(next.generation, initial.generation);
        assert_eq!(next.revision, initial.revision + 1);
        assert!(host.dispatch(&signal).is_err());
        assert!(host.edit(&initial, []).is_err());
        assert!(
            host.binding_feedback(
                &initial,
                PropertyId::new("text").unwrap(),
                PropertyValue::String("late".into())
            )
            .is_err()
        );
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&PropertyId::new("text").unwrap()],
            PropertyValue::String("edited".into())
        );
    }

    #[test]
    fn invalid_updates_are_atomic_and_remount_invalidates_old_tokens() {
        let mut host = host();
        let initial = host.mount(document()).unwrap();
        assert!(
            host.edit(
                &initial,
                [(
                    PropertyId::new("text").unwrap(),
                    PropertyValue::Boolean(true)
                )]
            )
            .is_err()
        );
        assert_eq!(&host.snapshot().unwrap().stamp, &initial);
        host.teardown(&initial).unwrap();
        assert!(host.render(&()).unwrap().is_none());
        let remount = host.mount(document()).unwrap();
        assert!(remount.generation > initial.generation);
        assert!(host.teardown(&initial).is_err());
        assert!(host.edit(&initial, []).is_err());
        let mut untrusted = document();
        untrusted.actions.insert(
            EventId::new("activate").unwrap(),
            ActionRef::new("arbitrary").unwrap(),
        );
        assert!(host.mount(untrusted).is_err());
        assert_eq!(&host.snapshot().unwrap().stamp, &remount);
    }
    #[test]
    fn registry_rejects_payload_events_and_unmapped_signals() {
        let existing = host();
        let mut schema = existing
            .catalog()
            .components
            .values()
            .next()
            .unwrap()
            .clone();
        let property = schema.properties.values().next().unwrap().clone();
        schema
            .events
            .values_mut()
            .next()
            .unwrap()
            .payload
            .insert(PropertyId::new("payload").unwrap(), property);
        let mut registry = PreviewRegistry::<()>::new();
        assert!(
            registry
                .register(schema, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut host = host();
        host.mount(document()).unwrap();
        let signal = host
            .snapshot()
            .unwrap()
            .signal(EventId::new("unmapped").unwrap());
        assert!(host.dispatch(&signal).is_err());
    }
}
