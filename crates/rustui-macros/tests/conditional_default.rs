#![deny(unused_doc_comments)]

extern crate self as rustui;

pub mod runtime {
    pub struct Action;

    impl Action {
        pub fn new<Component>(_: fn(&mut Component)) -> Self {
            Self
        }

        pub fn new_if_changed<Component>(_: fn(&mut Component) -> bool) -> Self {
            Self
        }
    }
}

use rustui_macros::component;

struct NoDefault;

#[component]
struct CfgUnavailable {
    #[cfg(any())]
    conditional: unavailable::Type<Self>,
    value: usize,
}

#[component]
struct CfgSelfReference {
    #[cfg(all())]
    recursive: Option<Box<Self>>,
}

#[component]
struct CfgDisabled<T> {
    #[cfg(any())]
    conditional: T,
    marker: core::marker::PhantomData<T>,
}

#[component]
struct CfgAttrDisabled<T> {
    #[cfg_attr(all(), cfg(any()))]
    conditional: T,
    marker: core::marker::PhantomData<T>,
}

#[component]
struct NestedCfgAttrDisabled<T> {
    #[cfg_attr(all(), cfg_attr(all(), cfg(any())))]
    conditional: T,
    marker: core::marker::PhantomData<T>,
}

#[component]
struct CfgEnabled<T> {
    #[cfg(all())]
    conditional: T,
    marker: core::marker::PhantomData<T>,
}

#[component]
struct CfgAttrDocumentation {
    #[cfg_attr(all(), doc = "documentation belongs to this field")]
    value: usize,
}

fn default_with_enabled_field<T: Default>() {
    let value: CfgEnabled<T> = Default::default();
    let _ = &value.conditional;
}

#[test]
fn cfg_removed_default_fields_do_not_bound_generic_type() {
    assert_eq!(CfgUnavailable::default().value, 0);
    assert!(CfgSelfReference::default().recursive.is_none());
    let _: CfgDisabled<NoDefault> = Default::default();
    let _: CfgAttrDisabled<NoDefault> = Default::default();
    let _: NestedCfgAttrDisabled<NoDefault> = Default::default();
    let documented = CfgAttrDocumentation::default();
    let _ = documented.value;
    default_with_enabled_field::<usize>();
}
