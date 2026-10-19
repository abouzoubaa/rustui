//! Popover ownership, inherited values, editing and release acceptance.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rustui::prelude::*;

use super::locale_selector::LocaleSelector;

pub fn demo() -> impl View {
    Component("popover-harness", PopoverHarness::default)
}

#[derive(Default)]
struct Probe {
    mounts: AtomicUsize,
    releases: AtomicUsize,
}

#[derive(Clone)]
struct InheritedVersion(u32);

#[component]
struct PopoverHarness {
    #[state(true)]
    present: bool,
    #[state(0)]
    inherited_version: u32,
    #[state(0)]
    observations: u32,
    probe: Arc<Probe>,
}

impl PopoverHarness {
    fn remove_owner(&mut self) {
        self.present = false;
    }
    fn restore_owner(&mut self) {
        self.present = true;
    }
    fn change_inherited(&mut self) {
        self.inherited_version += 1;
    }
    fn observe(&mut self) {
        self.observations += 1;
    }
}

impl View for PopoverHarness {
    fn body(&self) -> impl View {
        let probe = self.probe.clone();
        let present = self.present();
        VStack::new()
            .spacing(8.0)
            .child(Text("Popover ownership"))
            .child(
                Button(
                    format!(
                        "Observe: mounts {} / releases {}",
                        self.probe.mounts.load(Ordering::SeqCst),
                        self.probe.releases.load(Ordering::SeqCst)
                    ),
                    self.action(Self::observe),
                )
                .id("popover-observe"),
            )
            .child(
                Button("Restore popover owner", self.action(Self::restore_owner))
                    .id("popover-restore-owner")
                    .disabled(present),
            )
            .child(Environment(
                InheritedVersion(self.inherited_version()),
                OwnerActionReader::<Self, _, _>(move |owner| {
                    let probe = probe.clone();
                    let owner = owner.clone();
                    present.then(|| {
                        Component("popover-owner", move || {
                            PopoverOwner::new(probe.clone(), owner.clone())
                        })
                    })
                }),
            ))
    }
}

#[component]
struct PopoverOwner {
    #[state(false)]
    open: bool,
    #[state(0)]
    count: u32,
    #[state(String::new())]
    notes: String,
    probe: Arc<Probe>,
    harness: Option<OwnerAction<PopoverHarness>>,
}

impl PopoverOwner {
    fn new(probe: Arc<Probe>, harness: OwnerAction<PopoverHarness>) -> Self {
        Self {
            probe,
            harness: Some(harness),
            ..Self::default()
        }
    }
    fn increment(&mut self) {
        self.count += 1;
    }
    fn close(&mut self) {
        self.open = false;
    }
}

impl View for PopoverOwner {
    fn body(&self) -> impl View {
        let probe = self.probe.clone();
        let harness = self.harness.as_ref().expect("harness owner is supplied");
        VStack::new()
            .spacing(8.0)
            .child(Text(format!(
                "Open: {} · owner count: {} · notes: {}",
                self.open(),
                self.count(),
                self.notes()
            )))
            .child(Popover(
                "owner-popover",
                "Open editor popover",
                self.open_binding(),
                VStack::new()
                    .spacing(8.0)
                    .child(Text(format!("Owner count: {}", self.count())))
                    .child(TextField("Popover notes", self.notes_binding()).id("popover-notes"))
                    .child(
                        Button("Increment owner", self.action(Self::increment))
                            .id("popover-increment-owner"),
                    )
                    .child(EnvironmentReader::<InheritedVersion, _, _>(|value| {
                        Text(format!(
                            "Inherited version: {}",
                            value.map_or(0, |value| value.0)
                        ))
                    }))
                    .child(
                        Button(
                            "Change inherited value",
                            harness.action(PopoverHarness::change_inherited),
                        )
                        .id("popover-change-inherited"),
                    )
                    .child(Component("popover-child", move || {
                        PopoverChild::new(probe.clone())
                    }))
                    .child(LocaleSelector::new())
                    .child(LocalizedText("lab-navigation-locales"))
                    .child(Button("Close popover", self.action(Self::close)).id("popover-close"))
                    .child(
                        Button(
                            "Remove popover owner",
                            harness.action(PopoverHarness::remove_owner),
                        )
                        .id("popover-remove-owner"),
                    ),
            ))
    }
}

#[component]
struct PopoverChild {
    #[state(0)]
    count: u32,
    probe: Arc<Probe>,
}

impl PopoverChild {
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

impl View for PopoverChild {
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
        .id("popover-child-increment")
    }
}
