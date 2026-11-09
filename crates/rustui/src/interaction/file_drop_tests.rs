use super::*;
use crate::runtime::ComponentHost;
use gpui::{ExternalPaths, FileDropEvent, point};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rustui-file-drop-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, b"test").unwrap();
        path
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Zone;
impl View for Zone {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        div()
            .id("file-drop-zone")
            .debug_selector(|| "file-drop-zone".into())
            .w(px(200.))
            .h(px(100.))
            .child("Drop files")
            .into_any_element()
    }
}
struct Owner {
    dropped: Option<FileDrop>,
    writes: usize,
    enabled: bool,
    multiple: bool,
}
impl View for Owner {
    fn body(&self) -> impl View {
        let binding = Binding::new(
            self.dropped.clone(),
            |owner: &Self| owner.dropped.clone(),
            |owner, value| {
                owner.dropped = value;
                owner.writes += 1;
            },
        );
        DropTarget::files("images", binding, Zone)
            .extensions(["png", "jpg"])
            .multiple(self.multiple)
            .enabled(self.enabled)
    }
}
fn setup(
    cx: &mut gpui::TestAppContext,
    enabled: bool,
    multiple: bool,
) -> (
    gpui::Entity<ComponentHost<Owner>>,
    &mut gpui::VisualTestContext,
) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
            .unwrap();
    });
    let (owner, window) = cx.add_window_view(move |_, app| {
        ComponentHost::new(
            Owner {
                dropped: None,
                writes: 0,
                enabled,
                multiple,
            },
            app,
        )
    });
    window.update(|window, app| {
        window.activate_window();
        window.draw(app).clear(app);
    });
    (owner, window)
}
fn draw(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| {
        window.refresh();
        window.draw(app).clear(app);
    });
    window.run_until_parked();
}
fn enter(window: &mut gpui::VisualTestContext, paths: Vec<PathBuf>) {
    let position = window.debug_bounds("file-drop-zone").unwrap().center();
    window.simulate_event(FileDropEvent::Entered {
        position,
        paths: ExternalPaths(paths.into_iter().collect()),
    });
    draw(window);
}
fn submit(window: &mut gpui::VisualTestContext) {
    let position = window.debug_bounds("file-drop-zone").unwrap().center();
    window.simulate_event(FileDropEvent::Submit { position });
    draw(window);
}

#[gpui::test]
fn accepted_file_drop_writes_binding(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let image = files.path("image.png");
    let (owner, window) = setup(cx, true, true);
    enter(window, vec![image.clone()]);
    submit(window);
    window.update(|_, app| {
        let owner = owner.read(app).component();
        assert_eq!(owner.writes, 1);
        assert_eq!(
            owner.dropped,
            Some(FileDrop {
                accepted: vec![image],
                rejected: vec![]
            })
        );
    });
}
#[gpui::test]
fn mixed_file_drop_splits_accepted_and_rejected(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let image = files.path("image.png");
    let other = files.path("notes.txt");
    let (owner, window) = setup(cx, true, true);
    enter(window, vec![other.clone(), image.clone()]);
    submit(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().dropped,
            Some(FileDrop {
                accepted: vec![image],
                rejected: vec![other]
            })
        )
    });
}
#[gpui::test]
fn rejected_file_drop_writes_nothing(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let other = files.path("notes.txt");
    let (owner, window) = setup(cx, true, true);
    enter(window, vec![other]);
    submit(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().writes, 0);
        assert_eq!(owner.read(app).component().dropped, None);
    });
    assert!(window.debug_bounds("rustui-file-drop-rejected").is_none());
}
#[gpui::test]
fn file_extensions_are_case_insensitive(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let image = files.path("PHOTO.JpG");
    let (owner, window) = setup(cx, true, true);
    enter(window, vec![image.clone()]);
    submit(window);
    window.update(|_, app| {
        assert_eq!(
            owner
                .read(app)
                .component()
                .dropped
                .as_ref()
                .unwrap()
                .accepted,
            vec![image]
        )
    });
}
#[gpui::test]
fn single_file_drop_keeps_first_accepted(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let first = files.path("first.png");
    let second = files.path("second.jpg");
    let (owner, window) = setup(cx, true, false);
    enter(window, vec![first.clone(), second.clone()]);
    submit(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().dropped,
            Some(FileDrop {
                accepted: vec![first],
                rejected: vec![second]
            })
        )
    });
}
#[gpui::test]
fn disabled_file_drop_neither_highlights_nor_writes(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let image = files.path("image.png");
    let (owner, window) = setup(cx, false, true);
    enter(window, vec![image]);
    assert!(window.debug_bounds("rustui-file-drop-accepted").is_none());
    assert!(window.debug_bounds("rustui-file-drop-rejected").is_none());
    submit(window);
    window.update(|_, app| assert_eq!(owner.read(app).component().writes, 0));
}
#[gpui::test]
fn file_hover_distinguishes_accepted_and_rejected(cx: &mut gpui::TestAppContext) {
    let files = Files::new();
    let image = files.path("image.png");
    let other = files.path("notes.txt");
    let (_, window) = setup(cx, true, true);
    enter(window, vec![image]);
    assert!(window.debug_bounds("rustui-file-drop-accepted").is_some());
    window.simulate_event(FileDropEvent::Exited);
    draw(window);
    assert!(window.debug_bounds("rustui-file-drop-accepted").is_none());
    enter(window, vec![other]);
    assert!(window.debug_bounds("rustui-file-drop-rejected").is_some());
    submit(window);
    assert!(window.debug_bounds("rustui-file-drop-rejected").is_none());
}
#[gpui::test]
fn file_drop_accessibility_describes_extensions(cx: &mut gpui::TestAppContext) {
    let (_, window) = setup(cx, true, true);
    window.update(|window, app| {
        window.draw(app).clear(app);
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        let node = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|n| n["aria"]["label"] == "File drop target")
            .unwrap();
        assert_eq!(node["aria"]["description"], "Accepted file types: png, jpg");
    });
}

#[test]
fn custom_accept_composes_with_extensions_and_defaults_accept_all() {
    let binding = Binding::new(
        None,
        |owner: &Owner| owner.dropped.clone(),
        |owner, value| owner.dropped = value,
    );
    let all = DropTarget::files("all", binding.clone(), Zone);
    assert!(all.multiple && all.enabled);
    assert!((all.accept)(Path::new("note.txt")));
    let filtered = DropTarget::files("filtered", binding, Zone)
        .accept(|path| path.file_stem().is_some_and(|name| name == "keep"))
        .extensions(["png"]);
    assert!((filtered.accept)(Path::new("keep.PNG")));
    assert!(!(filtered.accept)(Path::new("skip.png")));
    assert!(!(filtered.accept)(Path::new("keep.txt")));
}

#[gpui::test]
fn described_file_drop_exposes_description_and_applies_predicate(cx: &mut gpui::TestAppContext) {
    struct DescribedOwner {
        dropped: Option<FileDrop>,
        writes: usize,
    }
    impl View for DescribedOwner {
        fn body(&self) -> impl View {
            DropTarget::files(
                "described-files",
                Binding::new(
                    self.dropped.clone(),
                    |owner: &Self| owner.dropped.clone(),
                    |owner, value| {
                        owner.dropped = value;
                        owner.writes += 1;
                    },
                ),
                Zone,
            )
            .accept_described(SharedString::from("files named keep"), |path| {
                path.file_stem().is_some_and(|name| name == "keep")
            })
        }
    }
    cx.update(gpui_component::init);
    let (owner, window) = cx.add_window_view(|_, app| {
        ComponentHost::new(
            DescribedOwner {
                dropped: None,
                writes: 0,
            },
            app,
        )
    });
    window.update(|window, app| {
        window.activate_window();
        window.draw(app).clear(app);
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        let node = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|node| node["aria"]["label"] == "File drop target")
            .unwrap();
        assert_eq!(node["aria"]["description"], "Accepts files named keep");
    });
    let files = Files::new();
    let keep_image = files.path("keep.png");
    let keep_text = files.path("keep.txt");
    let skip = files.path("skip.png");
    enter(
        window,
        vec![skip.clone(), keep_image.clone(), keep_text.clone()],
    );
    submit(window);
    window.update(|_, app| {
        let owner = owner.read(app).component();
        assert_eq!(owner.writes, 1);
        assert_eq!(
            owner.dropped,
            Some(FileDrop {
                accepted: vec![keep_image, keep_text],
                rejected: vec![skip],
            })
        );
    });
}

#[test]
fn described_accept_preserves_extension_composition_in_both_orders() {
    fn keep(path: &Path) -> bool {
        path.file_stem().is_some_and(|name| name == "keep")
    }
    for custom_first in [true, false] {
        let binding = Binding::new(None, |_: &()| None, |_: &mut (), _| {});
        let legacy = DropTarget::files("legacy", binding.clone(), Zone);
        let described = DropTarget::files("described", binding, Zone);
        let (legacy, described) = if custom_first {
            (
                legacy.accept(keep).extensions(["png"]),
                described
                    .accept_described("files named keep", keep)
                    .extensions(["png"]),
            )
        } else {
            (
                legacy.extensions(["png"]).accept(keep),
                described
                    .extensions(["png"])
                    .accept_described("files named keep", keep),
            )
        };
        for (name, before_extensions, after_extensions) in [
            ("keep.PNG", true, true),
            ("skip.png", false, false),
            ("keep.txt", false, true),
            ("keep", false, true),
        ] {
            let expected = if custom_first {
                before_extensions
            } else {
                after_extensions
            };
            assert_eq!(
                (legacy.accept)(Path::new(name)),
                expected,
                "legacy: {name}, custom_first={custom_first}"
            );
            assert_eq!(
                (described.accept)(Path::new(name)),
                expected,
                "described: {name}, custom_first={custom_first}"
            );
        }
        assert_eq!(
            described.description,
            if custom_first {
                "Accepted file types: png"
            } else {
                "Accepts files named keep"
            }
        );
    }
}
