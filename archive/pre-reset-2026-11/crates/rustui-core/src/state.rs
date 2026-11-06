//! Borrowed state access backed by a field on the owning component.

use crate::Binding;

/// Read access to a component field together with its two-way binding.
pub struct State<'a, T> {
    value: &'a T,
    binding: Binding<T>,
}

impl<'a, T> State<'a, T> {
    #[doc(hidden)]
    pub fn new(value: &'a T, binding: Binding<T>) -> Self {
        Self { value, binding }
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.clone()
    }

    pub fn value(&self) -> &T {
        self.value
    }

    pub fn binding(&self) -> Binding<T> {
        self.binding.clone()
    }
}
