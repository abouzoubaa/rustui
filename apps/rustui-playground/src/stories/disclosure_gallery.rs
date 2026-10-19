//! Controlled disclosure and hidden-content acceptance using public RustUI APIs.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("disclosure-gallery", DisclosureGallery::default)
}

#[derive(Default)]
struct Probe {
    mounts: AtomicUsize,
    releases: AtomicUsize,
}

#[component]
struct DisclosureGallery {
    #[state(false)]
    open: bool,
    #[state(false)]
    disabled: bool,
    #[state(String::new())]
    note: String,
    #[state(0)]
    observations: u32,
    probe: Arc<Probe>,
}

impl DisclosureGallery {
    fn force_open(&mut self) {
        self.open = true;
    }

    fn force_close(&mut self) {
        self.open = false;
    }

    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }

    fn observe(&mut self) {
        self.observations += 1;
    }
}

impl View for DisclosureGallery {
    fn body(&self) -> impl View {
        let probe = self.probe.clone();
        VStack::new()
            .spacing(8.0)
            .child(Text("Disclosure control"))
            .child(Text(format!(
                "Open: {} · disabled: {} · note: {}",
                self.open(),
                self.disabled(),
                self.note()
            )))
            .child(
                Button(
                    format!(
                        "Observe: mounts {} / releases {}",
                        self.probe.mounts.load(Ordering::SeqCst),
                        self.probe.releases.load(Ordering::SeqCst),
                    ),
                    self.action(Self::observe),
                )
                .id("disclosure-observe"),
            )
            .child(
                Disclosure(
                    "disclosure-gallery-details",
                    "Details",
                    self.open_binding(),
                    VStack::new()
                        .spacing(6.0)
                        .child(Text("This content mounts only while expanded."))
                        .child(
                            TextField("Disclosure note", self.note_binding())
                                .id("disclosure-gallery-note"),
                        )
                        .child(Component("disclosure-child", move || {
                            DisclosureChild::new(probe.clone())
                        })),
                )
                .disabled(self.disabled()),
            )
            .child(Button("Open externally", self.action(Self::force_open)).id("disclosure-open"))
            .child(
                Button("Close externally", self.action(Self::force_close)).id("disclosure-close"),
            )
            .child(
                Button("Toggle disabled", self.action(Self::toggle_disabled))
                    .id("disclosure-disabled"),
            )
    }
}

#[component]
struct DisclosureChild {
    #[state(0)]
    count: u32,
    probe: Arc<Probe>,
}

impl DisclosureChild {
    fn new(probe: Arc<Probe>) -> Self {
        Self {
            probe,
            ..Self::default()
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for DisclosureChild {
    fn on_mount(&mut self, _cx: &mut rustui::App) {
        self.probe.mounts.fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _cx: &mut rustui::App) {
        self.probe.releases.fetch_add(1, Ordering::SeqCst);
    }

    fn body(&self) -> impl View {
        Button(
            format!("Child count: {}", self.count()),
            self.action(Self::increment),
        )
        .id("disclosure-child-increment")
    }
}
