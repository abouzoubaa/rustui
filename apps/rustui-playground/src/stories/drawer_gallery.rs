//! Canonical Drawer story and compiled-local factory, using only public RustUI APIs.
use rustui::{localization::MessageArguments, prelude::*, runtime::Action};

pub fn demo() -> impl View {
    specimen_with_localized_demo(
        "drawer-gallery".into(),
        "Open Drawer".into(),
        "Details".into(),
        String::new(),
    )
}
pub fn specimen(id: String, trigger: String, title: String, body: String) -> impl View {
    specimen_inner(id, trigger, title, body, false)
}

fn specimen_with_localized_demo(
    id: String,
    trigger: String,
    title: String,
    body: String,
) -> impl View {
    specimen_inner(id, trigger, title, body, true)
}

fn specimen_inner(
    id: String,
    trigger: String,
    title: String,
    body: String,
    localize_demo: bool,
) -> impl View {
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
        localize_demo,
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
    localize_demo: bool,
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
        let mut trigger = Button(
            self.trigger.clone(),
            Action::new(|owner: &mut Self| owner.visible = !owner.visible),
        )
        .id("drawer-toggle");
        if self.localize_demo {
            trigger = trigger.localized_label("lab-drawer-open");
        }
        let mut view = VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-drawer-heading"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(trigger)
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
                        .localized_label("lab-drawer-next-edge")
                        .id("drawer-edge"),
                    )
                    .child(
                        Button(
                            "Unmount/remount",
                            Action::new(|owner: &mut Self| owner.mounted = !owner.mounted),
                        )
                        .localized_label("lab-drawer-mount")
                        .id("drawer-mounted"),
                    ),
            )
            .child(LocalizedText(if self.visible {
                "lab-drawer-requested-open"
            } else {
                "lab-drawer-requested-closed"
            }))
            .child(LocalizedText(match self.edge {
                DrawerEdge::Left => "lab-drawer-edge-left",
                DrawerEdge::Right => "lab-drawer-edge-right",
                DrawerEdge::Top => "lab-drawer-edge-top",
                DrawerEdge::Bottom => "lab-drawer-edge-bottom",
            }))
            .child(LocalizedText(match self.error {
                None => "lab-drawer-error-none",
                Some(DrawerError::ModalConflict) => "lab-drawer-error-conflict",
                Some(DrawerError::MissingRootHost) => "lab-drawer-error-root",
            }))
            .child(LocalizedText("lab-drawer-activations").with_arguments(
                MessageArguments::new().with("count", self.activations.to_string()),
            ))
            .child(
                Sheet("drawer-competing-sheet", "Open competing Sheet")
                    .localized_trigger("lab-drawer-competing-sheet")
                    .title("Competing modal")
                    .localized_title("lab-drawer-competing-title")
                    .content(LocalizedText("lab-drawer-exclusive-help")),
            )
            .child(
                Dialog("drawer-competing-dialog", "Open competing Dialog")
                    .localized_trigger("lab-drawer-competing-dialog")
                    .title("Competing modal")
                    .localized_title("lab-drawer-competing-title")
                    .content(LocalizedText("lab-drawer-exclusive-help")),
            );
        if self.mounted {
            let mut drawer = Drawer("details-drawer", self.visible())
                .edge(self.edge)
                .size(self.size)
                .title(self.title.clone())
                .accessibility_label("Details drawer")
                .localized_close_label("rustui-action-close")
                .presentation_error(self.error());
            if self.localize_demo {
                drawer = drawer.localized_title(if self.title == "Updated details" {
                    "lab-drawer-title-updated"
                } else {
                    "lab-drawer-title"
                });
            }
            view = view.child(
                drawer.content(
                    VStack::new()
                        .spacing(8.0)
                        .child(
                            TextField("Notes", self.notes())
                                .localized_placeholder("lab-drawer-notes")
                                .id("drawer-notes"),
                        )
                        .child(LocalizedText("lab-drawer-current-notes").with_arguments(
                            MessageArguments::new().with("notes", self.notes.clone()),
                        ))
                        .child(
                            Button(
                                "Update title and size",
                                Action::new(|owner: &mut Self| {
                                    owner.title = "Updated details".into();
                                    owner.size = if owner.size == 350.0 { 420.0 } else { 350.0 };
                                }),
                            )
                            .localized_label("lab-drawer-refresh")
                            .id("drawer-refresh"),
                        )
                        .child(
                            Button(
                                "Owner close",
                                Action::new(|owner: &mut Self| owner.visible = false),
                            )
                            .localized_label("lab-drawer-owner-close")
                            .id("drawer-owner-close"),
                        )
                        .child(
                            Button(
                                "Release Drawer",
                                Action::new(|owner: &mut Self| owner.mounted = false),
                            )
                            .localized_label("lab-drawer-release")
                            .id("drawer-release"),
                        )
                        .child(
                            Button(
                                "Activate",
                                Action::new(|owner: &mut Self| owner.activations += 1),
                            )
                            .localized_label("lab-drawer-activate")
                            .id("drawer-activate"),
                        ),
                ),
            );
        }
        view
    }
}
