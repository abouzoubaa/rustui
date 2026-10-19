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
                    panic!("Action dispatched with the wrong component owner type");
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
                    panic!("Action dispatched with the wrong component owner type");
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
