//! Native geometry, deferred ownership, retained state and branch cleanup.

use rustui::{component, prelude::*};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub fn demo() -> impl View {
    Component("geometry-gallery", GeometryGallery::default)
}

#[derive(Default)]
struct Counts {
    mounts: AtomicUsize,
    releases: AtomicUsize,
    wide_mounts: AtomicUsize,
    wide_releases: AtomicUsize,
}

#[component]
struct GeometryGallery {
    #[state(true)]
    wide: bool,
    #[state(true)]
    mounted: bool,
    #[state(String::from("Root-owned text"))]
    name: String,
    #[state(0)]
    root_hits: usize,
    #[state(0)]
    samples: usize,
    counts: Arc<Counts>,
}

impl GeometryGallery {
    fn toggle_width(&mut self) {
        self.wide = !self.wide;
    }
    fn toggle_mount(&mut self) {
        self.mounted = !self.mounted;
    }
    fn hit(&mut self) {
        self.root_hits += 1;
    }
    fn observe(&mut self) {
        self.samples += 1;
    }
    fn replace(&mut self) {
        self.name = format!("Root replacement {}", self.root_hits);
    }
}

impl View for GeometryGallery {
    fn body(&self) -> impl View {
        let counts = self.counts.clone();
        let binding = self.name_binding();
        let action = self.action(Self::hit);
        let reader = if self.mounted() {
            AnyView::new(
                GeometryReader::new("responsive-reader", move |size| {
                    let local_counts = counts.clone();
                    let wide_counts = counts.clone();
                    let branch = if size.width >= 420.0 {
                        AnyView::new(Component("wide-only", move || GeometryCounter {
                            counts: wide_counts.clone(),
                            wide_only: true,
                            ..GeometryCounter::default()
                        }))
                    } else {
                        AnyView::new(Text("Narrow branch: wide-only counter removed"))
                    };
                    VStack::new()
                        .spacing(6.0)
                        .child(Text(format!(
                            "Native size {:.0} × {:.0}",
                            size.width, size.height
                        )))
                        .child(LocalizedText("rustui-action-close"))
                        .child(TextField("Root-owned binding", binding.clone()))
                        .child(Button("Root callback +", action.clone()).id("geometry-root-action"))
                        .child(Component("persistent-counter", move || GeometryCounter {
                            counts: local_counts.clone(),
                            ..GeometryCounter::default()
                        }))
                        .child(branch)
                })
                .width(if self.wide() { 540.0 } else { 280.0 })
                .height(240.0),
            )
        } else {
            AnyView::new(Text("Reader omitted; reinsert creates fresh child owners."))
        };
        VStack::new().spacing(8.0)
            .child(Text("GeometryReader: explicit native bounds"))
            .child(super::locale_selector::LocaleSelector::new())
            .child(HStack::new().spacing(6.0)
                .child(Button("Toggle width", self.action(Self::toggle_width)).id("geometry-width"))
                .child(Button("Remove / reinsert", self.action(Self::toggle_mount)).id("geometry-mount"))
                .child(Button("Sample lifetimes", self.action(Self::observe)).id("geometry-sample")))
            .child(Button("Replace bound text externally", self.action(Self::replace)).id("geometry-replace"))
            .child(Text(format!("Root hits {} · bound text: {}", self.root_hits(), self.name())))
            .child(Text(format!("Samples {} · persistent mounts/releases {}/{} · wide mounts/releases {}/{}",
                self.samples(), self.counts.mounts.load(Ordering::SeqCst), self.counts.releases.load(Ordering::SeqCst),
                self.counts.wide_mounts.load(Ordering::SeqCst), self.counts.wide_releases.load(Ordering::SeqCst))))
            .child(Text("Resize: persistent count stays. Narrow removes wide counter. Remove/reinsert resets both. Locale selection also checks inherited direction and translation."))
            .child(reader)
    }
}

#[component]
struct GeometryCounter {
    counts: Arc<Counts>,
    wide_only: bool,
    #[state(0)]
    count: usize,
}

impl GeometryCounter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for GeometryCounter {
    fn on_mount(&mut self, _: &mut rustui::App) {
        if self.wide_only {
            &self.counts.wide_mounts
        } else {
            &self.counts.mounts
        }
        .fetch_add(1, Ordering::SeqCst);
    }
    fn on_unmount(&mut self, _: &mut rustui::App) {
        if self.wide_only {
            &self.counts.wide_releases
        } else {
            &self.counts.releases
        }
        .fetch_add(1, Ordering::SeqCst);
    }
    fn body(&self) -> impl View {
        Button(
            format!(
                "{} count {}",
                if self.wide_only {
                    "Wide-only"
                } else {
                    "Persistent"
                },
                self.count()
            ),
            self.action(Self::increment),
        )
        .id(if self.wide_only {
            "geometry-wide-count"
        } else {
            "geometry-local-count"
        })
    }
}
