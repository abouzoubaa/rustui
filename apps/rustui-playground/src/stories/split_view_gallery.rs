//! Native resizing and keyed panel lifetime review through public RustUI APIs.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

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
            .child(Text("SplitView: drag native dividers, increment panes, then reorder"))
            .child(Text(format!("Vertical: {} · reversed: {} · B present: {} · narrow limits: {} · observations: {}", vertical, self.reverse(), self.middle(), narrow_limits, self.observations())))
            .child(Text(format!("Ranges (pt): A 60..{a_max}, B 60..800, C 60..800")))
            .child(Text(lifecycle))
            .child(Container(split).width(560.0).height(300.0).border())
            .child(HStack::new().spacing(6.0)
                .child(Button("Switch axis", self.action(Self::toggle_axis)).id("split-axis"))
                .child(Button("Reverse panes", self.action(Self::toggle_order)).id("split-reverse"))
                .child(Button("Remove / insert B", self.action(Self::toggle_middle)).id("split-middle")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Narrow / restore limits", self.action(Self::toggle_limits)).id("split-limits"))
                .child(Button("Observe lifecycle", self.action(Self::observe)).id("split-observe")))
            .child(Text("Reorder preserves each counter. Narrow A without changing panel IDs, then restore its range. Removing B releases it; reinserting B starts at zero. Axis changes reset size preferences."))
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
                .child(Text(format!("Pane {}", ["A", "B", "C"][self.index])))
                .child(
                    Button(
                        format!("Count: {}", self.count()),
                        self.action(Self::increment),
                    )
                    .id("increment"),
                ),
        )
        .fill()
        .padding(8.0)
        .background(Surface::Panel)
    }
}
