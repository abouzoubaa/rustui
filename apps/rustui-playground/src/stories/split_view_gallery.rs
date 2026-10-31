//! Native resizing and keyed panel lifetime review through public RustUI APIs.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rustui::localization::MessageArguments;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("split-view-gallery", SplitGallery::default)
}

#[derive(Default)]
struct Lifecycle {
    mounts: [AtomicUsize; 3],
    releases: [AtomicUsize; 3],
}

#[component]
struct SplitGallery {
    #[state(false)]
    vertical: bool,
    #[state(false)]
    reverse: bool,
    #[state(true)]
    middle: bool,
    #[state(false)]
    narrow_limits: bool,
    #[state(0)]
    observations: u32,
    lifecycle: Arc<Lifecycle>,
}

impl SplitGallery {
    fn toggle_axis(&mut self) {
        self.vertical = !self.vertical;
    }
    fn toggle_order(&mut self) {
        self.reverse = !self.reverse;
    }
    fn toggle_middle(&mut self) {
        self.middle = !self.middle;
    }
    fn toggle_limits(&mut self) {
        self.narrow_limits = !self.narrow_limits;
    }
    fn observe(&mut self) {
        self.observations += 1;
    }
}

impl View for SplitGallery {
    fn body(&self) -> impl View {
        let vertical = self.vertical();
        let narrow_limits = self.narrow_limits();
        let a_max = if narrow_limits {
            if vertical { 70.0 } else { 120.0 }
        } else {
            800.0
        };
        let mut indices = vec![0, 1, 2];
        if !self.middle() {
            indices.retain(|index| *index != 1);
        }
        if self.reverse() {
            indices.reverse();
        }
        let panels = indices.into_iter().map(|index| {
            let lifecycle = self.lifecycle.clone();
            let id = ["A", "B", "C"][index];
            let initial = if vertical {
                [80.0, 100.0, 120.0][index]
            } else {
                [160.0, 180.0, 220.0][index]
            };
            ResizablePanel::new(
                id,
                Component("counter", move || PaneCounter {
                    index,
                    lifecycle: lifecycle.clone(),
                    ..PaneCounter::default()
                }),
            )
            .initial_size(initial)
            .size_range(60.0..if index == 0 { a_max } else { 800.0 })
        });
        let split = if vertical {
            SplitView::vertical("review-split")
        } else {
            SplitView::horizontal("review-split")
        }
        .panels(panels);
        let lifecycle = (0..3)
            .map(|index| {
                format!(
                    "{} mounts {} / releases {}",
                    ["A", "B", "C"][index],
                    self.lifecycle.mounts[index].load(Ordering::SeqCst),
                    self.lifecycle.releases[index].load(Ordering::SeqCst)
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");
        VStack::new().spacing(8.0)
            .child(LocalizedText("lab-split-heading"))
            .child(Text(format!("Vertical: {} · reversed: {} · B present: {} · narrow limits: {} · observations: {}", vertical, self.reverse(), self.middle(), narrow_limits, self.observations())))
            .child(Text(format!("Ranges (pt): A 60..{a_max}, B 60..800, C 60..800")))
            .child(Text(lifecycle))
            .child(Container(split).width(560.0).height(300.0).border())
            .child(HStack::new().spacing(6.0)
                .child(Button("Switch axis", self.action(Self::toggle_axis)).localized_label("lab-split-axis").id("split-axis"))
                .child(Button("Reverse panes", self.action(Self::toggle_order)).localized_label("lab-split-reverse").id("split-reverse"))
                .child(Button("Remove / insert B", self.action(Self::toggle_middle)).localized_label("lab-split-middle").id("split-middle")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Narrow / restore limits", self.action(Self::toggle_limits)).localized_label("lab-split-limits").id("split-limits"))
                .child(Button("Observe lifecycle", self.action(Self::observe)).localized_label("lab-split-observe").id("split-observe")))
            .child(LocalizedText("lab-split-help"))
    }
}

#[component]
struct PaneCounter {
    index: usize,
    lifecycle: Arc<Lifecycle>,
    #[state(0)]
    count: u32,
}

impl PaneCounter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for PaneCounter {
    fn on_mount(&mut self, _: &mut rustui::App) {
        self.lifecycle.mounts[self.index].fetch_add(1, Ordering::SeqCst);
    }
    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.lifecycle.releases[self.index].fetch_add(1, Ordering::SeqCst);
    }
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .spacing(6.0)
                .child(LocalizedText("lab-split-pane").with_arguments(
                    MessageArguments::new().with("name", ["A", "B", "C"][self.index]),
                ))
                .child(
                    Button(
                        format!("Count: {}", self.count()),
                        self.action(Self::increment),
                    )
                    .localized_label_with_arguments(
                        "lab-split-count",
                        MessageArguments::new().with("count", i64::from(self.count())),
                    )
                    .id("increment"),
                ),
        )
        .fill()
        .padding(8.0)
        .background(Surface::Panel)
    }
}

#[cfg(test)]
mod tests {
    use rustui::localization::{Locale, MessageArguments, format_integer};

    #[test]
    fn split_story_messages_resolve_with_arguments_in_runtime_locales() {
        let catalogs = super::super::locale_selector::catalogs();
        for language in ["en", "fr", "ar"] {
            let locale = Locale::parse(language).unwrap();
            for key in [
                "lab-split-heading",
                "lab-split-axis",
                "lab-split-reverse",
                "lab-split-middle",
                "lab-split-limits",
                "lab-split-observe",
                "lab-split-help",
            ] {
                let text = catalogs
                    .format(&locale, key, &MessageArguments::new())
                    .unwrap();
                assert!(!text.trim().is_empty() && text != key, "{language}: {key}");
            }
            let pane = catalogs
                .format(
                    &locale,
                    "lab-split-pane",
                    &MessageArguments::new().with("name", "B"),
                )
                .unwrap();
            assert!(pane.contains('B'), "{language}: {pane}");
            let count = catalogs
                .format(
                    &locale,
                    "lab-split-count",
                    &MessageArguments::new().with("count", 42_i64),
                )
                .unwrap();
            assert!(
                count.contains(&format_integer(42, &locale)),
                "{language}: {count}"
            );
        }
    }
}
