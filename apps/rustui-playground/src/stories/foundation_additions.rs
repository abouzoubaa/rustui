//! Playground examples for the current theme reload and layout primitives.

#[cfg(not(target_family = "wasm"))]
use std::env;
use std::{
    cell::RefCell,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    rc::{Rc, Weak},
    time::Duration,
};

use rustui::{
    prelude::*,
    theme::{AppearanceMode, ThemeFile, registry::active_theme},
};

#[cfg(target_family = "wasm")]
use rustui::theme::{activate_theme, parse_theme};

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(module = "/web/theme-reload.js")]
extern "C" {
    type WebThemeRequest;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = startThemeFetch)]
    fn start_theme_fetch() -> WebThemeRequest;
    #[wasm_bindgen::prelude::wasm_bindgen(method, getter)]
    fn promise(this: &WebThemeRequest) -> js_sys::Promise;
    #[wasm_bindgen::prelude::wasm_bindgen(method)]
    fn cancel(this: &WebThemeRequest);
}

#[cfg(target_family = "wasm")]
struct ThemeRequestGuard(WebThemeRequest);

#[cfg(target_family = "wasm")]
impl Drop for ThemeRequestGuard {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

#[cfg(target_family = "wasm")]
async fn load_web_theme() -> Result<String, String> {
    let request = ThemeRequestGuard(start_theme_fetch());
    wasm_bindgen_futures::JsFuture::from(request.0.promise())
        .await
        .map_err(|error| error.as_string().unwrap_or_else(|| format!("{error:?}")))?
        .as_string()
        .ok_or_else(|| "Theme response is not text".to_owned())
}

const RELOAD_INTERVAL: Duration = Duration::from_millis(350);
#[cfg(not(target_family = "wasm"))]
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
    action_epoch: u64,
    task: Option<WatchTask>,
    #[cfg(target_family = "wasm")]
    manual_task: Option<WatchTask>,
    #[cfg(target_family = "wasm")]
    manual_pending: bool,
    #[cfg(target_family = "wasm")]
    request_generation: u64,
    #[cfg(target_family = "wasm")]
    web_source: Option<String>,
    #[cfg(target_family = "wasm")]
    web_mode: Option<AppearanceMode>,
}

struct ReloadLease {
    state: Weak<RefCell<ReloadState>>,
    epoch: u64,
}

impl ReloadLease {
    fn current(&self) -> Option<Rc<RefCell<ReloadState>>> {
        let state = self.state.upgrade()?;
        let current = state.borrow().action_epoch == self.epoch;
        current.then_some(state)
    }
}

impl ReloadState {
    fn set_result(&mut self, result: String) -> bool {
        if self.result == result {
            return false;
        }
        self.result = result;
        true
    }

    #[cfg(not(target_family = "wasm"))]
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
        #[cfg(target_family = "wasm")]
        {
            self.manual_task.take();
            self.manual_pending = false;
            self.next_request();
        }
    }

    #[cfg(target_family = "wasm")]
    fn next_request(&mut self) -> u64 {
        self.request_generation = self
            .request_generation
            .checked_add(1)
            .expect("theme request generation exhausted");
        self.request_generation
    }

    #[cfg(target_family = "wasm")]
    fn apply_web(
        &mut self,
        source: Result<String, String>,
        mode: AppearanceMode,
        cx: &mut rustui::App,
        manual: bool,
    ) -> bool {
        let source = match source {
            Ok(source) => source,
            Err(error) => {
                self.had_error = true;
                return self.set_result(format!("Reload error: {error}"));
            }
        };
        if self.web_source.as_deref() == Some(source.as_str()) && self.web_mode == Some(mode) {
            return if std::mem::take(&mut self.had_error) {
                self.set_result(format!(
                    "Theme data recovered; {mode:?} appearance unchanged."
                ))
            } else if manual {
                self.set_result(format!("No change; {mode:?} appearance already active."))
            } else {
                false
            };
        }
        let result = parse_theme(&source)
            .map_err(|error| error.to_string())
            .and_then(|document| {
                activate_theme(&document, Some(mode), cx).map_err(|error| error.to_string())
            });
        match result {
            Ok(()) => {
                self.web_source = Some(source);
                self.web_mode = Some(mode);
                self.had_error = false;
                self.set_result(format!(
                    "Applied {mode:?} appearance from browser theme data."
                ));
                true
            }
            Err(error) => {
                self.had_error = true;
                self.set_result(format!("Reload error: {error}"))
            }
        }
    }

    fn unmount(&mut self) {
        self.stop();
        self.action_epoch = self
            .action_epoch
            .checked_add(1)
            .expect("theme action epoch exhausted");
    }
}

impl FoundationAdditions {
    fn new() -> Self {
        #[cfg(target_family = "wasm")]
        let path = PathBuf::from("themes/macos27.json");
        #[cfg(not(target_family = "wasm"))]
        let path = env::var_os(THEME_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::playground_resource_path("themes/macos27.json"));
        #[cfg(not(target_family = "wasm"))]
        let (theme_file, result) = match ThemeFile::open(&path) {
            Ok(theme_file) => (
                Some(theme_file),
                format!("Ready to reload {}.", path.display()),
            ),
            Err(error) => (None, format!("Open error: {error}")),
        };
        #[cfg(target_family = "wasm")]
        let (theme_file, result) = (None, "Ready to load browser theme data.".to_owned());
        let had_error = theme_file.is_none();
        Self {
            reload: Rc::new(RefCell::new(ReloadState {
                theme_file,
                result,
                had_error,
                enabled: false,
                generation: 0,
                action_epoch: 0,
                task: None,
                #[cfg(target_family = "wasm")]
                manual_task: None,
                #[cfg(target_family = "wasm")]
                manual_pending: false,
                #[cfg(target_family = "wasm")]
                request_generation: 0,
                #[cfg(target_family = "wasm")]
                web_source: None,
                #[cfg(target_family = "wasm")]
                web_mode: None,
            })),
            path,
        }
    }

    fn reload_action(&self, mode: AppearanceMode) -> AppAction {
        let lease = self.reload_lease();
        let path = self.path.clone();
        AppAction::new(move |cx| {
            let Some(reload) = lease.current() else {
                return;
            };
            #[cfg(not(target_family = "wasm"))]
            if reload.borrow_mut().check(&path, mode, cx, true) {
                cx.refresh_windows();
            }
            #[cfg(target_family = "wasm")]
            {
                let request = {
                    let mut state = reload.borrow_mut();
                    state.manual_pending = true;
                    state.next_request()
                };
                let weak = Rc::downgrade(&reload);
                let epoch = lease.epoch;
                let task = cx.spawn(async move |cx| {
                    let source = load_web_theme().await;
                    let Some(reload) = weak.upgrade() else {
                        return;
                    };
                    cx.update(|cx| {
                        let mut state = reload.borrow_mut();
                        if state.action_epoch != epoch || state.request_generation != request {
                            return;
                        }
                        state.manual_pending = false;
                        if state.apply_web(source, mode, cx, true) {
                            cx.refresh_windows();
                        }
                    });
                });
                let mut state = reload.borrow_mut();
                state.manual_task = Some(Box::pin(task));
                state.set_result("Loading browser theme data.".to_owned());
                cx.refresh_windows();
            }
        })
    }

    fn toggle_auto_action(&self) -> AppAction {
        let lease = self.reload_lease();
        let path = self.path.clone();
        AppAction::new(move |cx| {
            let Some(reload) = lease.current() else {
                return;
            };
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
                    #[cfg(not(target_family = "wasm"))]
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
                    #[cfg(target_family = "wasm")]
                    let keep_watching = {
                        if !reload.borrow().enabled || reload.borrow().generation != generation {
                            break;
                        }
                        let next = cx.update(|cx| {
                            let mut state = reload.borrow_mut();
                            if !state.enabled
                                || state.generation != generation
                                || state.manual_pending
                            {
                                return None;
                            }
                            let mode = active_theme(cx).map(|theme| theme.mode)?;
                            Some((state.next_request(), mode))
                        });
                        drop(reload);
                        let Some((request, _mode)) = next else {
                            continue;
                        };
                        let source = load_web_theme().await;
                        let Some(reload) = weak.upgrade() else {
                            break;
                        };
                        cx.update(|cx| {
                            let mut state = reload.borrow_mut();
                            if !state.enabled || state.generation != generation {
                                return false;
                            }
                            if state.request_generation == request {
                                if let Some(mode) = active_theme(cx).map(|theme| theme.mode) {
                                    if state.apply_web(source, mode, cx, false) {
                                        cx.refresh_windows();
                                    }
                                }
                            }
                            true
                        })
                    };
                    if !keep_watching {
                        break;
                    }
                }
            });
            reload.borrow_mut().task = Some(Box::pin(task));
            cx.refresh_windows();
        })
    }

    fn reload_lease(&self) -> ReloadLease {
        ReloadLease {
            state: Rc::downgrade(&self.reload),
            epoch: self.reload.borrow().action_epoch,
        }
    }
}

impl View for FoundationAdditions {
    fn on_unmount(&mut self, _: &mut rustui::App) {
        self.reload.borrow_mut().unmount();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_actions_do_not_keep_owner_alive_or_survive_unmount() {
        let owner = FoundationAdditions::new();
        let weak = Rc::downgrade(&owner.reload);
        let stale = owner.reload_lease();
        let _manual = owner.reload_action(AppearanceMode::Light);
        let _automatic = owner.toggle_auto_action();
        assert_eq!(Rc::strong_count(&owner.reload), 1);
        assert!(stale.current().is_some());

        owner.reload.borrow_mut().unmount();
        assert!(stale.current().is_none());
        assert!(owner.reload_lease().current().is_some());

        drop(owner);
        assert!(weak.upgrade().is_none());
    }
}
