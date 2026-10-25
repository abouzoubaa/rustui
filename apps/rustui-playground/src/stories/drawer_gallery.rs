//! Canonical Drawer story and compiled-local factory, using only public RustUI APIs.
use rustui::{prelude::*, runtime::Action};

pub fn demo() -> impl View {
    specimen(
        "drawer-gallery".into(),
        "Open Drawer".into(),
        "Details".into(),
        "Editable owner notes".into(),
    )
}
pub fn specimen(id: String, trigger: String, title: String, body: String) -> impl View {
    Component(id, move || DrawerGallery {
        visible: false,
        mounted: true,
        edge: DrawerEdge::Right,
        size: 350.0,
        error: None,
        notes: body.clone(),
        title: title.clone(),
        trigger: trigger.clone(),
        activations: 0,
    })
}
struct DrawerGallery {
    visible: bool,
    mounted: bool,
    edge: DrawerEdge,
    size: f32,
    error: Option<DrawerError>,
    notes: String,
    title: String,
    trigger: String,
    activations: usize,
}
impl DrawerGallery {
    fn visible(&self) -> Binding<bool> {
        Binding::new(
            self.visible,
            |owner: &Self| owner.visible,
            |owner, v| owner.visible = v,
        )
    }
    fn error(&self) -> Binding<Option<DrawerError>> {
        Binding::new(
            self.error,
            |owner: &Self| owner.error,
            |owner, v| owner.error = v,
        )
    }
    fn notes(&self) -> Binding<String> {
        Binding::new(
            self.notes.clone(),
            |owner: &Self| owner.notes.clone(),
            |owner, v| owner.notes = v,
        )
    }
}
impl View for DrawerGallery {
    fn body(&self) -> impl View {
        let mut view = VStack::new()
            .spacing(8.0)
            .child(Text(
                "Drawer: controlled visibility and window-owned modal hosting",
            ))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button(
                            self.trigger.clone(),
                            Action::new(|owner: &mut Self| owner.visible = !owner.visible),
                        )
                        .id("drawer-toggle"),
                    )
                    .child(
                        Button(
                            "Next edge",
                            Action::new(|owner: &mut Self| {
                                owner.edge = match owner.edge {
                                    DrawerEdge::Left => DrawerEdge::Top,
                                    DrawerEdge::Top => DrawerEdge::Right,
                                    DrawerEdge::Right => DrawerEdge::Bottom,
                                    DrawerEdge::Bottom => DrawerEdge::Left,
                                }
                            }),
                        )
                        .id("drawer-edge"),
                    )
                    .child(
                        Button(
                            "Unmount/remount",
                            Action::new(|owner: &mut Self| owner.mounted = !owner.mounted),
                        )
                        .id("drawer-mounted"),
                    ),
            )
            .child(Text(format!(
                "Requested open: {} | Edge: {:?} | Error: {:?} | Activations: {}",
                self.visible, self.edge, self.error, self.activations
            )))
            .child(
                Sheet("drawer-competing-sheet", "Open competing Sheet")
                    .title("Competing modal")
                    .content(Text(
                        "Drawer is exclusive; native Sheet/Dialog behavior is preserved.",
                    )),
            )
            .child(
                Dialog("drawer-competing-dialog", "Open competing Dialog")
                    .title("Competing modal")
                    .content(Text(
                        "Drawer is exclusive; native Sheet/Dialog behavior is preserved.",
                    )),
            );
        if self.mounted {
            view = view.child(
                Drawer("details-drawer", self.visible())
                    .edge(self.edge)
                    .size(self.size)
                    .title(self.title.clone())
                    .accessibility_label("Details drawer")
                    .localized_close_label("rustui-action-close")
                    .presentation_error(self.error())
                    .content(
                        VStack::new()
                            .spacing(8.0)
                            .child(TextField("Notes", self.notes()).id("drawer-notes"))
                            .child(Text(format!("Current notes: {}", self.notes)))
                            .child(
                                Button(
                                    "Update title and size",
                                    Action::new(|owner: &mut Self| {
                                        owner.title = "Updated details".into();
                                        owner.size =
                                            if owner.size == 350.0 { 420.0 } else { 350.0 };
                                    }),
                                )
                                .id("drawer-refresh"),
                            )
                            .child(
                                Button(
                                    "Owner close",
                                    Action::new(|owner: &mut Self| owner.visible = false),
                                )
                                .id("drawer-owner-close"),
                            )
                            .child(
                                Button(
                                    "Release Drawer",
                                    Action::new(|owner: &mut Self| owner.mounted = false),
                                )
                                .id("drawer-release"),
                            )
                            .child(
                                Button(
                                    "Activate",
                                    Action::new(|owner: &mut Self| owner.activations += 1),
                                )
                                .id("drawer-activate"),
                            ),
                    ),
            );
        }
        view
    }
}
