//! Typed two-way access to state retained by a component owner.

use std::{any::Any, rc::Rc};

type Read<T> = dyn Fn(&dyn Any) -> Option<T>;
type Write<T> = dyn Fn(&mut dyn Any, T) -> bool;

/// A typed field lens whose storage remains in the owning component.
pub struct Binding<T> {
    current: Rc<T>,
    read: Rc<Read<T>>,
    write: Rc<Write<T>>,
}

impl<T: 'static> Binding<T> {
    #[doc(hidden)]
    pub fn new<Component: 'static>(
        current: T,
        read: fn(&Component) -> T,
        write: fn(&mut Component, T),
    ) -> Self {
        Self {
            current: Rc::new(current),
            read: Rc::new(move |owner| owner.downcast_ref::<Component>().map(read)),
            write: Rc::new(move |owner, value| {
                let Some(owner) = owner.downcast_mut::<Component>() else {
                    panic!("Binding written with the wrong component owner type");
                };
                write(owner, value);
                true
            }),
        }
    }

    /// Captures immutable local context for a typed owner binding.
    ///
    /// The write callback reports whether its update was accepted. This allows
    /// controls to reject a callback belonging to an old mounted revision.
    /// Context remains an ordinary Rust value and is never serialized.
    pub fn new_with<Component: 'static, Context: Clone + 'static>(
        current: T,
        context: Context,
        read: fn(&Component, &Context) -> T,
        write: fn(&mut Component, &Context, T) -> bool,
    ) -> Self {
        let read_context = context.clone();
        Self {
            current: Rc::new(current),
            read: Rc::new(move |owner| {
                owner
                    .downcast_ref::<Component>()
                    .map(|owner| read(owner, &read_context))
            }),
            write: Rc::new(move |owner, value| {
                let owner = owner
                    .downcast_mut::<Component>()
                    .expect("Binding written with the wrong component owner type");
                write(owner, &context, value)
            }),
        }
    }

    /// Returns the value captured while the owning component described its body.
    pub fn value(&self) -> T
    where
        T: Clone,
    {
        self.current.as_ref().clone()
    }

    #[doc(hidden)]
    pub fn read_from(&self, owner: &dyn Any) -> Option<T> {
        (self.read)(owner)
    }

    #[doc(hidden)]
    pub fn write_to(&self, owner: &mut dyn Any, value: T) -> bool {
        (self.write)(owner, value)
    }

    /// Reads the owner's current value and writes it back when the update reports a change.
    #[doc(hidden)]
    pub fn update_on(&self, owner: &mut dyn Any, update: impl FnOnce(&mut T) -> bool) -> bool {
        let mut value = self
            .read_from(owner)
            .unwrap_or_else(|| panic!("Binding updated with the wrong component owner type"));
        if !update(&mut value) {
            return false;
        }
        self.write_to(owner, value)
    }
}

impl<T> Clone for Binding<T> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            read: self.read.clone(),
            write: self.write.clone(),
        }
    }
}

#[cfg(test)]
mod captured_context_tests {
    use super::Binding;

    struct Owner {
        revision: u64,
        text: String,
    }

    #[test]
    fn rejects_old_captured_context_without_mutating_owner() {
        let mut owner = Owner {
            revision: 1,
            text: "before".into(),
        };
        let binding = Binding::new_with(
            owner.text.clone(),
            1_u64,
            |owner: &Owner, _| owner.text.clone(),
            |owner: &mut Owner, revision, value| {
                if owner.revision != *revision {
                    return false;
                }
                owner.text = value;
                owner.revision += 1;
                true
            },
        );
        assert!(binding.write_to(&mut owner, "accepted".into()));
        assert!(!binding.write_to(&mut owner, "stale".into()));
        assert_eq!(owner.text, "accepted");
        assert_eq!(owner.revision, 2);
    }
}
