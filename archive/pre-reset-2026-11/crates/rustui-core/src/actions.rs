use std::{any::Any, rc::Rc};

use crate::Invalidation;

/// A typed component method that can be invoked by a rendered control.
#[derive(Clone)]
pub struct Action {
    invoke: Rc<dyn Fn(&mut dyn Any) -> Invalidation>,
}

impl Action {
    /// Wraps a component mutation method for dispatch by a RustUI view host.
    #[doc(hidden)]
    pub fn new<Component: 'static>(action: fn(&mut Component)) -> Self {
        Self {
            invoke: Rc::new(move |target| {
                let Some(component) = target.downcast_mut::<Component>() else {
                    return Invalidation::None;
                };
                action(component);
                Invalidation::Notify
            }),
        }
    }

    /// Wraps a mutation that reports whether it changed observable state.
    #[doc(hidden)]
    pub fn new_if_changed<Component: 'static>(action: fn(&mut Component) -> bool) -> Self {
        Self {
            invoke: Rc::new(move |target| {
                let Some(component) = target.downcast_mut::<Component>() else {
                    return Invalidation::None;
                };
                action(component).into()
            }),
        }
    }

    /// Dispatches this action when the supplied component has the matching type.
    #[doc(hidden)]
    pub fn dispatch(&self, target: &mut dyn Any) -> Invalidation {
        (self.invoke)(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Owner {
        value: usize,
    }

    struct WrongOwner {
        mutation_count: usize,
    }

    fn increment(owner: &mut Owner) {
        owner.value += 1;
    }

    fn increment_if_changed(owner: &mut Owner) -> bool {
        owner.value += 1;
        true
    }

    fn unchanged(_: &mut Owner) -> bool {
        false
    }

    #[test]
    fn matching_owner_dispatch_reports_mutation_and_change_contracts() {
        let action = Action::new(increment);
        let changed_action = Action::new_if_changed(increment_if_changed);
        let unchanged_action = Action::new_if_changed(unchanged);
        let mut owner = Owner { value: 0 };

        assert_eq!(action.dispatch(&mut owner), Invalidation::Notify);
        assert_eq!(owner.value, 1);
        assert_eq!(unchanged_action.dispatch(&mut owner), Invalidation::None);
        assert_eq!(owner.value, 1);
        assert_eq!(changed_action.dispatch(&mut owner), Invalidation::Notify);
        assert_eq!(owner.value, 2);
    }

    #[test]
    fn wrong_owner_dispatch_is_a_noop_for_both_action_kinds() {
        let action = Action::new(increment);
        let conditional_action = Action::new_if_changed(increment_if_changed);
        let mut wrong_owner = WrongOwner { mutation_count: 0 };

        assert_eq!(action.dispatch(&mut wrong_owner), Invalidation::None);
        assert_eq!(
            conditional_action.dispatch(&mut wrong_owner),
            Invalidation::None
        );
        assert_eq!(wrong_owner.mutation_count, 0);
    }
}
