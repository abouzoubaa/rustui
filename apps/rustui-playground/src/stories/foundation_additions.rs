//! Playground examples for the current theme reload and layout primitives.

use std::{
    cell::RefCell,
    env,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    rc::Rc,
    time::Duration,
};

use rustui::{
    prelude::*,
    theme::{AppearanceMode, ThemeFile, registry::active_theme},
};

const RELOAD_INTERVAL: Duration = Duration::from_millis(350);
const THEME_PATH_ENV: &str = "RUSTUI_THEME_RELOAD_PATH";
type WatchTask = Pin<Box<dyn Future<Output = ()>>>;

/// Returns a retained, compact composition for the normal Layout story.
pub fn foundation_additions() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(Component(
            "m15-foundation-additions",
            FoundationAdditions::new,
        ))
        .child(super::theme_inspector::demo())
}

struct FoundationAdditions {
    reload: Rc<RefCell<ReloadState>>,
    path: PathBuf,
}

struct ReloadState {
    theme_file: Option<ThemeFile>,
    result: String,
    had_error: bool,
    enabled: bool,
    generation: u64,
    task: Option<WatchTask>,
}

impl ReloadState {
    fn set_result(&mut self, result: String) -> bool {
        if self.result == result {
            return false;
        }
        self.result = result;
        true
    }

    fn check(
        &mut self,
        path: &Path,
        mode: AppearanceMode,
        cx: &mut rustui::App,
        manual: bool,
    ) -> bool {
        if self.theme_file.is_none() {
            match ThemeFile::open(path) {
                Ok(theme_file) => self.theme_file = Some(theme_file),
                Err(error) => {
                    self.had_error = true;
                    return self.set_result(format!("Open error: {error}"));
                }
            }
        }
        let result = self
            .theme_file
            .as_mut()
            .expect("theme file opened")
            .reload_if_changed(Some(mode), cx);
        match result {
            Ok(true) => {
                self.had_error = false;
                self.set_result(format!("Applied {mode:?} appearance from file."));
                true
            }
            Ok(false) if std::mem::take(&mut self.had_error) => self.set_result(format!(
                "Theme file recovered; {mode:?} appearance is unchanged."
            )),
            Ok(false) if manual => {
                self.set_result(format!("No change; {mode:?} appearance is already active."))
            }
            Ok(false) => false,
            Err(error) => {
                self.had_error = true;
                self.set_result(format!("Reload error: {error}"))
            }
        }
    }

    fn stop(&mut self) {
        self.enabled = false;
        self.generation = self
            .generation
            .checked_add(1)
            .expect("theme watcher generation exhausted");
        self.task.take();
    }
}

impl FoundationAdditions {
    fn new() -> Self {
        let path = env::var_os(THEME_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join("themes/macos27.json")
            });
        let (theme_file, result) = match ThemeFile::open(&path) {
            Ok(theme_file) => (
                Some(theme_file),
                format!("Ready to reload {}.", path.display()),
            ),
            Err(error) => (None, format!("Open error: {error}")),
        };
        let had_error = theme_file.is_none();
        Self {
            reload: Rc::new(RefCell::new(ReloadState {
                theme_file,
                result,
                had_error,
                enabled: false,
                generation: 0,
                task: None,
            })),
            path,
        }
    }

    fn reload_action(&self, mode: AppearanceMode) -> AppAction {
        let reload = Rc::clone(&self.reload);
        let path = self.path.clone();
        AppAction::new(move |cx| {
            if reload.borrow_mut().check(&path, mode, cx, true) {
                cx.refresh_windows();
            }
        })
    }

    fn toggle_auto_action(&self) -> AppAction {
        let reload = Rc::clone(&self.reload);
        let path = self.path.clone();
        AppAction::new(move |cx| {
            let generation = {
                let mut state = reload.borrow_mut();
                if state.enabled {
                    state.stop();
                    state.set_result("Automatic theme reload disabled.".to_owned());
                    cx.refresh_windows();
                    return;
                }
                state.enabled = true;
                state.generation = state
                    .generation
                    .checked_add(1)
                    .expect("theme watcher generation exhausted");
                state.set_result("Watching theme file every 350 ms.".to_owned());
                state.generation
            };
            let weak = Rc::downgrade(&reload);
            let path = path.clone();
            let task = cx.spawn(async move |cx| {
                loop {
                    cx.background_executor().timer(RELOAD_INTERVAL).await;
                    let Some(reload) = weak.upgrade() else {
                        break;
                    };
                    let keep_watching = cx.update(|cx| {
                        let mut state = reload.borrow_mut();
                        if !state.enabled || state.generation != generation {
                            return false;
                        }
                        let Some(mode) = active_theme(cx).map(|theme| theme.mode) else {
                            return true;
                        };
                        if state.check(&path, mode, cx, false) {
                            cx.refresh_windows();
                        }
                        true
                    });
                    if !keep_watching {
                        break;
                    }
                }
            });
            reload.borrow_mut().task = Some(Box::pin(task));
            cx.refresh_windows();
        })
    }
}

impl View for FoundationAdditions {
    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.reload.borrow_mut().stop();
    }

    fn body(&self) -> impl View {
        let insets = LogicalInsets::new(4.0, 4.0, 16.0, 4.0).expect("fixed logical insets");
        let ltr = insets.resolve(LayoutDirection::LeftToRight);
        let rtl = insets.resolve(LayoutDirection::RightToLeft);
        let state = self.reload.borrow();
        let result = state.result.clone();
        let enabled = state.enabled;
        drop(state);

        Container(
            VStack::new()
                .spacing(5.0)
                .child(Text("Theme and layout"))
                .child(
                    HStack::new()
                        .spacing(4.0)
                        .child(
                            Button("Light", self.reload_action(AppearanceMode::Light))
                                .accessibility_label(format!(
                                    "Reload light appearance. Last result: {result}"
                                )),
                        )
                        .child(
                            Button("Dark", self.reload_action(AppearanceMode::Dark))
                                .accessibility_label(format!(
                                    "Reload dark appearance. Last result: {result}"
                                )),
                        ),
                )
                .child(
                    Button(
                        if enabled { "Stop auto" } else { "Start auto" },
                        self.toggle_auto_action(),
                    )
                    .accessibility_label(format!(
                        "{} automatic theme file reload. Last result: {result}",
                        if enabled { "Disable" } else { "Enable" },
                    )),
                )
                .child(Text(result).font_size(12.0))
                .child(Divider())
                .child(Text("Flexible HStack spacer"))
                .child(
                    HStack::new()
                        .fill()
                        .spacing(4.0)
                        .child(Text("Leading"))
                        .child(Spacer().min_length(SpacerAxis::Horizontal, 24.0))
                        .child(Text("Trailing")),
                )
                .child(
                    HStack::new()
                        .spacing(6.0)
                        .child(Text("First side"))
                        .child(Divider().vertical())
                        .child(Text("Second side")),
                )
                .child(Text(format!("LTR left/right: {}/{}", ltr.left, ltr.right)))
                .child(Text(format!("RTL left/right: {}/{}", rtl.left, rtl.right))),
        )
        .width(200.0)
        .padding(8.0)
        .border()
    }
}
