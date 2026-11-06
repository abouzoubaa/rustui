//! Typed same-process drag/drop and keyboard-equivalent model actions.

use rustui::prelude::*;
use std::{thread, time::Duration};

pub fn demo() -> impl View {
    Component(
        "drag-drop-owner-release-probe",
        DragDropOwnerReleaseProbe::default,
    )
}

#[component]
struct DragDropOwnerReleaseProbe {
    #[state(true)]
    gallery_visible: bool,
    #[state(String::from("Ready. Arm removal, then begin and hold a drag."))]
    status: String,
}

impl DragDropOwnerReleaseProbe {
    fn finish_removal(&mut self, result: AsyncState<(), String>) -> bool {
        match result {
            AsyncState::Loading => self.status = "Removal armed; start and hold a drag.".into(),
            AsyncState::Success(()) => {
                self.gallery_visible = false;
                self.status = "Gallery owner removed; release the drag now.".into();
            }
            AsyncState::Error(error) => self.status = format!("Removal probe failed: {error}"),
            AsyncState::Idle => return false,
        }
        true
    }

    fn restore(&mut self) {
        self.gallery_visible = true;
        self.status = "Gallery restored. Arm removal, then begin and hold a drag.".into();
    }
}

impl View for DragDropOwnerReleaseProbe {
    fn body(&self) -> impl View {
        let gallery_visible = self.gallery_visible();
        let status = self.status();
        VStack::new()
			.spacing(8.0)
			.child(Text("M66: remove the drag gallery component owner during a held native drag."))
			.child(Text("Arm, immediately drag a source over the target, and keep the pointer held until removal is reported."))
			.child(OwnerTaskReader(move |tasks: OwnerTask<DragDropOwnerReleaseProbe>| {
				VStack::new()
					.spacing(6.0)
					.child(Button(
						"Arm owner removal (3 seconds)",
						tasks.start(
							"remove-drag-owner",
							|| async {
								thread::sleep(Duration::from_secs(3));
								Ok::<(), String>(())
							},
							DragDropOwnerReleaseProbe::finish_removal,
						),
					)
					.disabled(!gallery_visible)
					.id("drag-owner-release-arm"))
					.child(OwnerActionReader(
						|owner: OwnerAction<DragDropOwnerReleaseProbe>| {
							Button("Restore gallery", owner.action(DragDropOwnerReleaseProbe::restore))
								.id("drag-owner-release-restore")
						},
					))
			}))
			.child(Text(status))
			.child(gallery_visible.then(|| {
				Component("drag-drop-gallery-owner", DragDropGallery::default)
			}))
			.child((!gallery_visible).then(|| {
				Container(Text("Drag gallery owner has been released."))
					.width(360.0)
					.height(180.0)
					.padding(8.0)
					.border()
			}))
    }
}

#[component]
struct DragDropGallery {
    #[state(vec!["A".to_owned(), "B".to_owned(), "C".to_owned()])]
    sources: Vec<String>,
    #[state(Vec::<String>::new())]
    destination: Vec<String>,
    #[state(false)]
    transfer: bool,
    #[state(true)]
    show_target: bool,
    #[state(true)]
    source_enabled: bool,
    #[state(true)]
    target_enabled: bool,
    #[state(None::<DropEvent<String>>)]
    state_drop: Option<DropEvent<String>>,
    #[state(String::from("No drop yet."))]
    result: String,
}

impl DragDropGallery {
    fn read_drop(&self) -> Option<DropEvent<String>> {
        None
    }

    fn write_drop(&mut self, event: Option<DropEvent<String>>) {
        let Some(event) = event else {
            return;
        };
        if self.show_target {
            self.apply_drop(event);
        }
    }

    fn apply_drop(&mut self, event: DropEvent<String>) {
        let value = event.value;
        match event.operation {
            DragOperation::Copy => {
                self.destination.push(value.clone());
                self.result = format!("Copied {value} from {} to {}.", event.source, event.target);
            }
            DragOperation::Transfer => {
                if let Some(index) = self.sources.iter().position(|item| *item == value) {
                    self.sources.remove(index);
                    self.destination.push(value.clone());
                    self.result = format!(
                        "Transferred {value} from {} to {}.",
                        event.source, event.target
                    );
                } else {
                    self.result = format!("Transfer ignored: {value} is no longer in the source.");
                }
            }
        }
    }

    fn toggle_operation(&mut self) {
        self.transfer = !self.transfer;
    }

    fn toggle_target(&mut self) {
        self.show_target = !self.show_target;
    }

    fn finish_target_removal(&mut self, result: AsyncState<(), String>) -> bool {
        match result {
            AsyncState::Loading => {
                self.result = "Target removal armed; start and hold a drag.".into();
            }
            AsyncState::Success(()) => {
                self.show_target = false;
                self.result = "Target subtree removed; gallery owner remains mounted.".into();
            }
            AsyncState::Error(error) => {
                self.result = format!("Target removal probe failed: {error}");
            }
            AsyncState::Idle => return false,
        }
        true
    }

    fn toggle_source_enabled(&mut self) {
        self.source_enabled = !self.source_enabled;
    }

    fn toggle_target_enabled(&mut self) {
        self.target_enabled = !self.target_enabled;
    }

    fn reverse_sources(&mut self) {
        self.sources.reverse();
    }

    fn reset(&mut self) {
        self.sources = vec!["A".to_owned(), "B".to_owned(), "C".to_owned()];
        self.destination.clear();
        self.source_enabled = true;
        self.target_enabled = true;
        self.show_target = true;
        self.transfer = false;
        self.state_drop = None;
        self.result = "No drop yet.".to_owned();
    }

    fn copy_first(&mut self) {
        if let Some(value) = self.sources.first().cloned() {
            self.apply_drop(DropEvent {
                source: value.clone().into(),
                target: "gallery-target".into(),
                value,
                operation: DragOperation::Copy,
            });
        }
    }

    fn transfer_first(&mut self) {
        if let Some(value) = self.sources.first().cloned() {
            self.apply_drop(DropEvent {
                source: value.clone().into(),
                target: "gallery-target".into(),
                value,
                operation: DragOperation::Transfer,
            });
        }
    }
}

impl View for DragDropGallery {
    fn body(&self) -> impl View {
        let operation = if self.transfer() {
            DragOperation::Transfer
        } else {
            DragOperation::Copy
        };
        let mut sources = HStack::new().spacing(8.0);
        for value in self.sources() {
            sources = sources.child(Keyed(
                value.clone(),
                DragSource(
                    value.clone(),
                    value.clone(),
                    Container(Text(format!("Drag {value}")))
                        .width(84.0)
                        .height(42.0)
                        .padding(6.0)
                        .border(),
                )
                .operation(operation)
                .preview(format!("{operation:?} {value}"))
                .enabled(self.source_enabled()),
            ));
        }

        let dropped = Binding::new(None, Self::read_drop, Self::write_drop);
        let target = self.show_target().then(|| {
            DropTarget(
                "gallery-target",
                dropped,
                Container(Text("Drop String values here"))
                    .width(260.0)
                    .height(74.0)
                    .padding(8.0)
                    .border(),
            )
            .enabled(self.target_enabled())
        });
        let state_drop = self.state_drop().map_or_else(
            || "No component-state drop yet.".to_owned(),
            |event| {
                format!(
                    "Component state received {} from {}.",
                    event.value, event.source
                )
            },
        );

        VStack::new()
            .spacing(8.0)
            .child(Text("Native typed drag/drop · same process"))
            .child(Text(format!("Operation: {operation:?}. Drag A/B/C into the target.")))
            .child(OwnerTaskReader(move |tasks: OwnerTask<DragDropGallery>| {
                Button(
                    "Arm target removal (3 seconds)",
                    tasks.start(
                        "remove-drop-target-during-drag",
                        || async {
                            thread::sleep(Duration::from_secs(3));
                            Ok::<(), String>(())
                        },
                        DragDropGallery::finish_target_removal,
                    ),
                )
                .id("drag-target-removal-arm")
            }))
            .child(VStack::new()
                .spacing(6.0)
                .child(HStack::new().spacing(6.0)
                    .child(Button("Copy / Transfer", self.action(Self::toggle_operation)).id("drag-operation"))
                    .child(Button("Reverse sources", self.action(Self::reverse_sources)).id("drag-reverse"))
                    .child(Button("Enable / disable sources", self.action(Self::toggle_source_enabled)).id("drag-source-enabled")))
                .child(HStack::new().spacing(6.0)
                    .child(Button("Enable / disable target", self.action(Self::toggle_target_enabled)).id("drag-target-enabled"))
                    .child(Button("Remove / restore target", self.action(Self::toggle_target)).id("drag-target-toggle"))
                    .child(Button("Reset", self.action(Self::reset)).id("drag-reset"))))
            .child(Text(format!("Sources enabled: {} · target enabled: {}", self.source_enabled(), self.target_enabled())))
            .child(sources)
            .child(target)
            .child(
                DropTarget(
                    "gallery-state-target",
                    self.state_drop_binding(),
                    Container(Text("Drop here to store the typed event in component state"))
                        .height(54.0)
                        .padding(8.0)
                        .border(),
                )
                .enabled(self.target_enabled()),
            )
            .child(Text(state_drop))
            .child(DragSource("numeric-source", 42_usize, Text("Try dragging this number: incompatible type")).enabled(self.source_enabled()))
            .child(Text(format!("Destination: {:?}", self.destination())))
            .child(Text(self.result()))
            .child(Text("Keyboard alternative:"))
            .child(HStack::new().spacing(6.0)
                .child(Button("Copy first", self.action(Self::copy_first)).id("drag-copy-first"))
                .child(Button("Transfer first", self.action(Self::transfer_first)).id("drag-transfer-first")))
            .child(Text("Type filtering rejects the numeric source. Reversing preserves source IDs; removing the target during a drag must not dispatch to its released owner."))
    }
}
