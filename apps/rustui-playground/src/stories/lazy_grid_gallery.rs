//! LazyGrid scrolling, stable cell state, and root-owner callback probe.

use std::{
    num::NonZeroU16,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use rustui::prelude::*;

const INITIAL_ITEMS: usize = 2_400;
const TESTED_ID: usize = 0;

pub fn demo() -> impl View {
    Component("lazy-grid-gallery", LazyGridGallery::default)
}

fn initial_items() -> Vec<usize> {
    (0..INITIAL_ITEMS).collect()
}

#[derive(Default)]
struct Counts {
    factory_calls: AtomicUsize,
    mounts: AtomicUsize,
    releases: AtomicUsize,
    tested_mounts: AtomicUsize,
    tested_releases: AtomicUsize,
}

#[component]
struct LazyGridGallery {
    #[state(initial_items())]
    items: Vec<usize>,
    #[state(INITIAL_ITEMS)]
    next_id: usize,
    #[state(true)]
    show_grid: bool,
    #[state(0)]
    root_hits: usize,
    #[state(0)]
    observations: usize,
    counts: Arc<Counts>,
}

impl LazyGridGallery {
    fn reverse(&mut self) {
        self.items.reverse();
    }

    fn prepend(&mut self) {
        self.items.insert(0, self.next_id);
        self.next_id += 1;
    }

    fn remove_tested(&mut self) {
        self.items.retain(|id| *id != TESTED_ID);
    }

    fn restore_tested(&mut self) {
        if !self.items.contains(&TESTED_ID) {
            self.items.insert(0, TESTED_ID);
        }
    }

    fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    fn root_hit(&mut self) {
        self.root_hits += 1;
    }

    fn observe(&mut self) {
        self.observations += 1;
    }
}

impl View for LazyGridGallery {
    fn body(&self) -> impl View {
        let snapshot: Rc<[LazyGridItem<usize>]> = self
            .items()
            .into_iter()
            .map(|id| LazyGridItem::new(format!("item-{id:04}"), id))
            .collect::<Vec<_>>()
            .into();
        let counts = self.counts.clone();
        let root_action = self.action(Self::root_hit);
        let grid = self.show_grid().then(|| {
            LazyGrid::new(
                "gallery-lazy-grid",
                snapshot,
                NonZeroU16::new(3).unwrap(),
                260.0,
                90.0,
                move |index, id| {
                    counts.factory_calls.fetch_add(1, Ordering::SeqCst);
                    let counts = counts.clone();
                    let root_action = root_action.clone();
                    let id = *id;
                    VStack::new()
                        .spacing(2.0)
                        .child(Text(format!("#{index} · item-{id:04}")))
                        .child(Component("cell-owner", move || CellCounter {
                            id,
                            counts: counts.clone(),
                            ..CellCounter::default()
                        }))
                        .child(Button("Root +", root_action).id("root-hit"))
                },
            )
            .column_gap(8.0)
            .row_gap(6.0)
        });
        VStack::new()
            .spacing(8.0)
            .child(Text("LazyGrid · 2,400 stable IDs · three fixed columns"))
            .child(LocalizedText("rustui-action-close"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Reverse", self.action(Self::reverse)).id("lazy-reverse"))
                    .child(Button("Prepend", self.action(Self::prepend)).id("lazy-prepend"))
                    .child(Button("Remove item-0000", self.action(Self::remove_tested)).id("lazy-remove"))
                    .child(Button("Restore item-0000", self.action(Self::restore_tested)).id("lazy-restore")),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Hide / show grid", self.action(Self::toggle_grid)).id("lazy-show"))
                    .child(Button("Sample counts", self.action(Self::observe)).id("lazy-observe")),
            )
            .child(Text(format!(
                "Items {} · grid {} · root hits {} · samples {}",
                self.items().len(),
                self.show_grid(),
                self.root_hits(),
                self.observations()
            )))
            .child(Text(format!(
                "Factory {} · mounts {} · releases {} · item-0000 mounts {} / releases {}",
                self.counts.factory_calls.load(Ordering::SeqCst),
                self.counts.mounts.load(Ordering::SeqCst),
                self.counts.releases.load(Ordering::SeqCst),
                self.counts.tested_mounts.load(Ordering::SeqCst),
                self.counts.tested_releases.load(Ordering::SeqCst),
            )))
            .child(Text("Scroll, increment item-0000, reverse twice, and return: its local count should persist. Remove and restore item-0000: it should restart at zero. Root + must update root hits. Sample counts after scrolling to inspect lazy factory calls."))
            .child(grid)
    }
}

#[component]
struct CellCounter {
    id: usize,
    counts: Arc<Counts>,
    #[state(0)]
    count: usize,
}

impl CellCounter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for CellCounter {
    fn on_mount(&mut self, _: &mut rustui::App) {
        self.counts.mounts.fetch_add(1, Ordering::SeqCst);
        if self.id == TESTED_ID {
            self.counts.tested_mounts.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.counts.releases.fetch_add(1, Ordering::SeqCst);
        if self.id == TESTED_ID {
            self.counts.tested_releases.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn body(&self) -> impl View {
        Button(
            format!("Cell count {}", self.count()),
            self.action(Self::increment),
        )
        .id("cell-count")
    }
}
