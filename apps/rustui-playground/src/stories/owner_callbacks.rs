//! Two independent owners exercise explicit child-to-parent callback routing.

use rustui::prelude::*;

pub fn demo() -> impl View {
    HStack::new()
        .child(Component("callback-left", || Parent::named("Left")))
        .child(Component("callback-right", || Parent::named("Right")))
        .spacing(16.0)
}

#[component]
struct Parent {
    name: String,
    #[state(0)]
    count: i32,
}

impl Parent {
    fn named(name: &str) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for Parent {
    fn body(&self) -> impl View {
        let name = self.name.clone();
        let count = self.count();
        OwnerActionReader(move |target: OwnerAction<Parent>| {
            VStack::new()
                .child(Text(format!("{name} parent: {count}")))
                .child(
                    Component("callback-child", Child::default)
                        .props((target, name.clone(), count), Child::apply_props),
                )
                .spacing(6.0)
        })
    }
}

#[component]
struct Child {
    target: Option<OwnerAction<Parent>>,
    name: String,
    parent_count: i32,
    #[state(0)]
    local_count: i32,
}

impl Child {
    fn apply_props(&mut self, props: &(OwnerAction<Parent>, String, i32)) {
        self.target = Some(props.0.clone());
        self.name.clone_from(&props.1);
        self.parent_count = props.2;
    }

    fn increment_local(&mut self) {
        self.local_count += 1;
    }
}

impl View for Child {
    fn body(&self) -> impl View {
        let parent_button = self.target.as_ref().map(|target| {
            Button("Increment parent", target.action(Parent::increment))
                .id("callback-parent")
                .accessibility_label(format!(
                    "{} parent callback: {}",
                    self.name, self.parent_count
                ))
        });
        VStack::new()
            .child(parent_button)
            .child(
                Button("Increment local", self.action(Self::increment_local))
                    .id("callback-local")
                    .accessibility_label(format!(
                        "{} local callback: {}",
                        self.name,
                        self.local_count()
                    )),
            )
            .spacing(6.0)
    }
}
