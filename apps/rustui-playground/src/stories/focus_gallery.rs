//! Native focus-scope behavior through the public RustUI API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("focus-gallery", FocusGallery::default)
}

#[component]
struct FocusGallery {
    #[state(false)]
    scope_a_focused: bool,
    #[state(false)]
    scope_b_focused: bool,
    #[state(true)]
    scope_a_present: bool,
    #[state(String::new())]
    scope_a_text: String,
    #[state(String::new())]
    scope_b_text: String,
    #[state(String::new())]
    outside_text: String,
}

impl FocusGallery {
    fn request_a(&mut self) {
        self.scope_a_focused = true;
    }

    fn clear_a(&mut self) {
        self.scope_a_focused = false;
    }

    fn request_b(&mut self) {
        self.scope_b_focused = true;
    }

    fn remove_a(&mut self) {
        self.scope_a_focused = false;
        self.scope_a_present = false;
    }

    fn restore_a(&mut self) {
        self.scope_a_present = true;
    }
}

impl View for FocusGallery {
    fn body(&self) -> impl View {
        let scope_a = if self.scope_a_present() {
            AnyView::new(
                VStack::new()
                    .spacing(6.0)
                    .child(Text("Scope A"))
                    .child(
                        FocusScope(
                            "focus-gallery-scope-a",
                            self.scope_a_focused_binding(),
                            VStack::new()
                                .spacing(6.0)
                                .child(
                                    TextField("Scope A text field", self.scope_a_text_binding())
                                        .id("focus-a-text-field"),
                                )
                                .child(Component("focus-a-counter", || ScopeCounter::named("A"))),
                        )
                        .tab_stop(false)
                        .label("Programmatic focus target: Scope A container"),
                    )
                    .child(
                        Button("Remove Scope A", self.action(Self::remove_a)).id("focus-remove-a"),
                    ),
            )
        } else {
            AnyView::new(
                VStack::new()
                    .spacing(6.0)
                    .child(Text(
                        "Scope A is removed; restoring it creates a fresh child counter.",
                    ))
                    .child(
                        Button("Restore Scope A", self.action(Self::restore_a))
                            .id("focus-restore-a"),
                    ),
            )
        };

        VStack::new()
            .spacing(10.0)
            .child(Text("Native focus scopes"))
            .child(Text(
                "Request A and Request B target each scope container. The binding reports focus in that container or its descendants; it does not report a TextField's private text cursor.",
            ))
            .child(Text(format!(
                "Scope A container focused: {} · Scope B container focused: {}",
                self.scope_a_focused(),
                self.scope_b_focused(),
            )))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Button("Request A scope focus", self.action(Self::request_a)).id("focus-request-a"))
                    .child(Button("Clear A scope focus", self.action(Self::clear_a)).id("focus-clear-a"))
                    .child(Button("Request B scope focus", self.action(Self::request_b)).id("focus-request-b")),
            )
            .child(scope_a)
            .child(
                VStack::new()
                    .spacing(6.0)
                    .child(Text("Scope B"))
                    .child(FocusScope(
                        "focus-gallery-scope-b",
                        self.scope_b_focused_binding(),
                        VStack::new()
                            .spacing(6.0)
                            .child(TextField("Scope B text field", self.scope_b_text_binding()).id("focus-b-text-field"))
                            .child(Component("focus-b-counter", || ScopeCounter::named("B"))),
                    ).tab_stop(false).label("Programmatic focus target: Scope B container")),
            )
            .child(Text("Unrelated field outside both focus scopes"))
            .child(TextField("Outside focus test", self.outside_text_binding()).id("focus-outside-text-field"))
    }
}

#[component]
struct ScopeCounter {
    name: String,
    #[state(0)]
    count: u32,
}

impl ScopeCounter {
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

impl View for ScopeCounter {
    fn body(&self) -> impl View {
        HStack::new()
            .spacing(8.0)
            .child(Text(format!(
                "Scope {} child counter: {}",
                self.name,
                self.count()
            )))
            .child(
                Button(
                    format!("Increment {} child counter", self.name),
                    self.action(Self::increment),
                )
                .id("focus-scope-counter-increment"),
            )
    }
}
