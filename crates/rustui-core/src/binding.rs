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
