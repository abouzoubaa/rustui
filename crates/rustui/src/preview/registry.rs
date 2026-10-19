//! Explicit registration of ordinary compiled factories. No code is deserialized.
use std::{collections::BTreeMap, rc::Rc};

use crate::view::AnyView;

use super::host::PreviewSnapshot;
use super::{ActionRef, ComponentCatalog, ComponentId, ComponentSchema, PreviewError};

type Factory<C> = dyn Fn(&PreviewSnapshot, &C) -> AnyView;

/// A local catalog paired with compiled Rust factories and typed owner context.
///
/// `C` may contain ordinary Rust actions and bindings. It is never serialized.
/// Register all entries before handing the registry to a host.
pub struct PreviewRegistry<C> {
    catalog: ComponentCatalog,
    factories: BTreeMap<ComponentId, Rc<Factory<C>>>,
}

impl<C> Default for PreviewRegistry<C> {
    fn default() -> Self {
        Self {
            catalog: ComponentCatalog::new(),
            factories: BTreeMap::new(),
        }
    }
}

impl<C> PreviewRegistry<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a signal-only component and its compiled factory atomically.
    pub fn register(
        &mut self,
        schema: ComponentSchema,
        factory: impl Fn(&PreviewSnapshot, &C) -> AnyView + 'static,
    ) -> Result<(), PreviewError> {
        if schema
            .events
            .values()
            .any(|event| !event.payload.is_empty())
        {
            return Err(PreviewError::Invalid(
                "compiled preview events must be signal-only".into(),
            ));
        }
        let id = schema.id.clone();
        self.catalog.register_component(schema)?;
        self.factories.insert(id, Rc::new(factory));
        Ok(())
    }

    /// Explicitly allowlists a local action target. Handlers stay with the owner.
    pub fn register_action(&mut self, action: ActionRef) -> Result<(), PreviewError> {
        self.catalog.register_action(action)
    }

    pub fn catalog(&self) -> &ComponentCatalog {
        &self.catalog
    }

    pub(super) fn render(
        &self,
        snapshot: &PreviewSnapshot,
        context: &C,
    ) -> Result<AnyView, PreviewError> {
        let id = &snapshot.preview.document.component_id;
        let factory = self
            .factories
            .get(id)
            .ok_or_else(|| PreviewError::UnknownComponent(id.clone()))?;
        Ok(factory(snapshot, context))
    }
}
