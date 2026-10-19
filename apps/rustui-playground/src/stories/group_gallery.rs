//! Group flattening and keyed child lifetime through public RustUI APIs.

use std::{
    num::NonZeroU16,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("group-gallery", GroupGallery::default)
}

#[derive(Default)]
struct Lifetimes {
    mounts: [AtomicUsize; 3],
    releases: [AtomicUsize; 3],
}

#[component]
struct GroupGallery {
    #[state(false)]
    reverse: bool,
    #[state(true)]
    middle: bool,
    #[state(0)]
    samples: usize,
    lifetimes: Arc<Lifetimes>,
}

impl GroupGallery {
    fn toggle_order(&mut self) {
        self.reverse = !self.reverse;
    }

    fn toggle_middle(&mut self) {
        self.middle = !self.middle;
    }

    fn sample(&mut self) {
        self.samples += 1;
    }
}

impl View for GroupGallery {
    fn body(&self) -> impl View {
        let mut indices = vec![0, 1, 2];
        if !self.middle() {
            indices.retain(|index| *index != 1);
        }
        if self.reverse() {
            indices.reverse();
        }

        let mut children = Group::new().child(Text("Grouped, keyed children:"));
        for index in indices {
            let lifetimes = self.lifetimes.clone();
            let id = ["A", "B", "C"][index];
            children = children.child(Keyed(
                id,
                Component("group-counter", move || GroupCounter {
                    index,
                    lifetimes: lifetimes.clone(),
                    ..GroupCounter::default()
                }),
            ));
        }

        let lifetime_summary = (0..3)
            .map(|index| {
                format!(
                    "{}: mounts {} / releases {}",
                    ["A", "B", "C"][index],
                    self.lifetimes.mounts[index].load(Ordering::SeqCst),
                    self.lifetimes.releases[index].load(Ordering::SeqCst)
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");

        VStack::new()
            .spacing(10.0)
            .child(Text("Group: direct sibling placement"))
            .child(Text("Each pair below should have the same item order and layout."))
            .child(Text("HStack · direct / grouped"))
            .child(HStack::new().spacing(8.0).child(
                Container(HStack::new().spacing(4.0).child(Text("A")).child(Label("B")))
                    .padding(6.0)
                    .border(),
            ).child(
                Container(HStack::new().spacing(4.0).group(
                    Group::new().child(Text("A")).child(Label("B")),
                ))
                .padding(6.0)
                .border(),
            ))
            .child(Text("VStack · direct / grouped"))
            .child(HStack::new().spacing(8.0).child(
                Container(VStack::new().spacing(4.0).child(Text("A")).child(Label("B")))
                    .width(80.0)
                    .padding(6.0)
                    .border(),
            ).child(
                Container(VStack::new().spacing(4.0).group(
                    Group::new().child(Text("A")).child(Label("B")),
                ))
                .width(80.0)
                .padding(6.0)
                .border(),
            ))
            .child(Text("Grid · direct / grouped"))
            .child(HStack::new().spacing(8.0).child(
                Container(Grid::new(NonZeroU16::new(2).unwrap()).spacing(4.0)
                    .child(Text("A")).child(Label("B")))
                    .width(120.0).padding(6.0).border(),
            ).child(
                Container(Grid::new(NonZeroU16::new(2).unwrap()).spacing(4.0)
                    .group(Group::new().child(Text("A")).child(Label("B"))))
                    .width(120.0).padding(6.0).border(),
            ))
            .child(Text("ZStack · direct / grouped (B paints over A)"))
            .child(HStack::new().spacing(8.0).child(
                Container(ZStack::new().child(Text("A")).child(Label("B")))
                    .width(120.0).height(32.0).border(),
            ).child(
                Container(ZStack::new().group(
                    Group::new().child(Text("A")).child(Label("B")),
                ))
                .width(120.0).height(32.0).border(),
            ))
            .child(HStack::new().spacing(6.0)
                .child(Button("Reverse A/B/C", self.action(Self::toggle_order)).id("group-reverse"))
                .child(Button("Remove / restore B", self.action(Self::toggle_middle)).id("group-middle"))
                .child(Button("Sample lifetime", self.action(Self::sample)).id("group-sample")))
            .child(Text(format!("Samples {} · {lifetime_summary}", self.samples())))
            .child(HStack::new().spacing(6.0).group(children))
            .child(Text("Increment counts and edit fields, then reverse: surviving state should stay with A/B/C. Remove B, sample releases, restore B: its count and field restart."))
    }
}

#[component]
struct GroupCounter {
    index: usize,
    lifetimes: Arc<Lifetimes>,
    #[state(0)]
    count: usize,
    #[state(String::new())]
    note: String,
}

impl GroupCounter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for GroupCounter {
    fn on_mount(&mut self, _: &mut rustui::App) {
        self.lifetimes.mounts[self.index].fetch_add(1, Ordering::SeqCst);
    }

    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.lifetimes.releases[self.index].fetch_add(1, Ordering::SeqCst);
    }

    fn body(&self) -> impl View {
        let id = ["A", "B", "C"][self.index];
        Container(
            VStack::new()
                .spacing(4.0)
                .child(
                    Button(
                        format!("{id}: {}", self.count()),
                        self.action(Self::increment),
                    )
                    .id("increment"),
                )
                .child(TextField(format!("{id} note"), self.note_binding()).id("note")),
        )
        .width(135.0)
        .padding(6.0)
        .border()
    }
}
