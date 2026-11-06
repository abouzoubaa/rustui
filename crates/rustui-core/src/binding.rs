//! Typed two-way access to state retained by a component owner.

use std::{any::Any, rc::Rc};

type Read<T> = dyn Fn(&dyn Any) -> Option<T>;
type Write<T> = dyn Fn(&mut dyn Any, T) -> bool;

/// A typed field lens whose storage remains in the owning component.
///
/// [`Binding::value`] is the snapshot captured when the binding was created.
/// It does not change when the owner is updated; obtain a new binding on a
/// subsequent render to see the new snapshot. Binding-based update callbacks
/// read the current owner value rather than this snapshot.
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
                    return false;
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
                let Some(owner) = owner.downcast_mut::<Component>() else {
                    return false;
                };
                write(owner, &context, value)
            }),
        }
    }

    /// Returns the snapshot captured when this binding was created.
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
        let Some(mut value) = self.read_from(owner) else {
            return false;
        };
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

#[cfg(test)]
mod wrong_owner_write_tests {
    use super::Binding;

    struct Owner {
        value: String,
    }

    #[test]
    fn new_write_to_returns_false_for_wrong_owner_type() {
        let binding = Binding::new(
            String::from("initial"),
            |owner: &Owner| owner.value.clone(),
            |owner: &mut Owner, value| owner.value = value,
        );
        let mut wrong_owner = String::from("unchanged");

        assert!(!binding.write_to(&mut wrong_owner, String::from("updated")));
        assert_eq!(wrong_owner, "unchanged");
    }

    #[test]
    fn new_with_write_to_returns_false_for_wrong_owner_type() {
        let binding = Binding::new_with(
            String::from("initial"),
            (),
            |owner: &Owner, _| owner.value.clone(),
            |owner: &mut Owner, _context, value| {
                owner.value = value;
                true
            },
        );
        let mut wrong_owner = String::from("unchanged");

        assert!(!binding.write_to(&mut wrong_owner, String::from("updated")));
        assert_eq!(wrong_owner, "unchanged");
    }
}

#[cfg(test)]
mod update_on_tests {
    use super::Binding;

    struct Owner {
        value: i32,
    }

    #[test]
    fn wrong_owner_type_rejects_update_without_running_mutation() {
        let binding = Binding::new(
            7,
            |owner: &Owner| owner.value,
            |owner: &mut Owner, value| owner.value = value,
        );
        let mut wrong_owner = String::from("unchanged");

        let changed = binding.update_on(&mut wrong_owner, |_| {
            panic!("mutation must not run for the wrong owner type");
        });

        assert!(!changed);
        assert_eq!(wrong_owner, "unchanged");
    }

    #[test]
    fn no_op_update_does_not_write_or_report_a_change() {
        let mut owner = Owner { value: 7 };
        let binding = Binding::new(
            7,
            |owner: &Owner| owner.value,
            |owner: &mut Owner, value| owner.value = value,
        );

        assert!(!binding.update_on(&mut owner, |_| false));
        assert_eq!(owner.value, 7);
    }

    #[test]
    fn stale_context_rejects_update_without_mutating_owner() {
        struct VersionedOwner {
            revision: u64,
            value: i32,
        }

        let mut owner = VersionedOwner {
            revision: 2,
            value: 7,
        };
        let binding = Binding::new_with(
            7,
            1_u64,
            |owner: &VersionedOwner, _revision| owner.value,
            |owner: &mut VersionedOwner, revision, value| {
                if owner.revision != *revision {
                    return false;
                }
                owner.value = value;
                owner.revision += 1;
                true
            },
        );
        let mut update_ran = false;

        let changed = binding.update_on(&mut owner, |value| {
            update_ran = true;
            *value += 1;
            true
        });

        assert!(update_ran);
        assert!(!changed);
        assert_eq!(owner.revision, 2);
        assert_eq!(owner.value, 7);
    }

    #[test]
    fn consecutive_updates_read_the_latest_owner_value_in_order() {
        let mut owner = Owner { value: 7 };
        let binding = Binding::new(
            0,
            |owner: &Owner| owner.value,
            |owner: &mut Owner, value| owner.value = value,
        );

        for _ in 0..2 {
            assert!(binding.update_on(&mut owner, |value| {
                *value += 1;
                true
            }));
        }

        assert_eq!(owner.value, 9);
    }
}
