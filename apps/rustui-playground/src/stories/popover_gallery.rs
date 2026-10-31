//! Popover ownership, inherited values, editing and release acceptance.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rustui::{localization::MessageArguments, prelude::*};

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
        let mounts = self.probe.mounts.load(Ordering::SeqCst);
        let releases = self.probe.releases.load(Ordering::SeqCst);
        let args = MessageArguments::new()
            .with("mounts", mounts as i64)
            .with("releases", releases as i64);
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-popover-heading"))
            .child(LocaleSelector::new())
            .child(
                HStack::new().child(
                    Button(
                        format!("Observe: mounts {mounts} / releases {releases}"),
                        self.action(Self::observe),
                    )
                    .id("popover-observe")
                    .localized_label_with_arguments("lab-popover-observe", args),
                ),
            )
            .child(
                HStack::new().child(
                    Button("Restore popover owner", self.action(Self::restore_owner))
                        .id("popover-restore-owner")
                        .localized_label("lab-popover-restore")
                        .disabled(present),
                ),
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
        let count = self.count();
        let status_key = if self.open() {
            "lab-popover-open-status"
        } else {
            "lab-popover-closed-status"
        };
        let status = MessageArguments::new()
            .with("count", count as i64)
            .with("notes", self.notes());
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText(status_key).with_arguments(status))
            .child(
                HStack::new().child(
                    Popover(
                        "owner-popover",
                        "Open editor popover",
                        self.open_binding(),
                        VStack::new()
                            .spacing(8.0)
                            .child(LocalizedText("lab-popover-owner-count").with_arguments(
                                MessageArguments::new().with("count", count as i64),
                            ))
                            .child(
                                TextField("Popover notes", self.notes_binding())
                                    .localized_placeholder("lab-popover-notes")
                                    .id("popover-notes"),
                            )
                            .child(
                                HStack::new().child(
                                    Button("Increment owner", self.action(Self::increment))
                                        .localized_label("lab-popover-increment-owner")
                                        .id("popover-increment-owner"),
                                ),
                            )
                            .child(EnvironmentReader::<InheritedVersion, _, _>(|value| {
                                LocalizedText("lab-popover-inherited").with_arguments(
                                    MessageArguments::new().with(
                                        "version",
                                        i64::from(value.map_or(0, |value| value.0)),
                                    ),
                                )
                            }))
                            .child(
                                HStack::new().child(
                                    Button(
                                        "Change inherited value",
                                        harness.action(PopoverHarness::change_inherited),
                                    )
                                    .localized_label("lab-popover-change-inherited")
                                    .id("popover-change-inherited"),
                                ),
                            )
                            .child(Component("popover-child", move || {
                                PopoverChild::new(probe.clone())
                            }))
                            .child(LocaleSelector::new())
                            .child(LocalizedText("lab-navigation-locales"))
                            .child(
                                HStack::new().child(
                                    Button("Close popover", self.action(Self::close))
                                        .localized_label("lab-popover-close")
                                        .id("popover-close"),
                                ),
                            )
                            .child(
                                HStack::new().child(
                                    Button(
                                        "Remove popover owner",
                                        harness.action(PopoverHarness::remove_owner),
                                    )
                                    .localized_label("lab-popover-remove")
                                    .id("popover-remove-owner"),
                                ),
                            ),
                    )
                    .localized_label("lab-popover-open"),
                ),
            )
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
        let count = self.count();
        HStack::new().child(
            Button(
                format!("Child count: {count}"),
                self.action(Self::increment),
            )
            .localized_label_with_arguments(
                "lab-popover-child-count",
                MessageArguments::new().with("count", count as i64),
            )
            .id("popover-child-increment"),
        )
    }
}

#[cfg(test)]
mod tests {
    use rustui::localization::{Locale, MessageArguments, format_integer};

    #[test]
    fn popover_counts_format_in_supported_locales() {
        let catalogs = super::super::catalogs();
        for tag in ["en", "fr", "ar"] {
            let locale = Locale::parse(tag).unwrap();
            let args = MessageArguments::new()
                .with("count", 12_i64)
                .with("mounts", 12_i64)
                .with("releases", 34_i64)
                .with("notes", "example")
                .with("version", 12_i64);
            for key in [
                "lab-popover-observe",
                "lab-popover-open-status",
                "lab-popover-closed-status",
                "lab-popover-owner-count",
                "lab-popover-inherited",
                "lab-popover-child-count",
            ] {
                let result = catalogs.format(&locale, key, &args).unwrap();
                assert!(
                    result.contains(&format_integer(12, &locale)),
                    "{tag}: {key}: {result}"
                );
            }
        }
    }
}
