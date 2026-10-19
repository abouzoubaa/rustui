//! Typed values scoped to a RustUI component host.

use std::{any::Any, any::TypeId, collections::HashMap, rc::Rc};

/// Type-indexed environment values passed through RustUI rendering.
#[derive(Clone, Default)]
pub struct EnvironmentValues {
    values: HashMap<TypeId, Rc<dyn Any>>,
}

impl EnvironmentValues {
    /// Returns whether this scope contains any typed environment values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn set<T: 'static>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Rc::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }
}
