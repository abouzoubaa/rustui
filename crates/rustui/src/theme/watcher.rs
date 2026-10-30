//! Explicit, transactional reloads for a RustUI theme file.

use std::{error::Error, fmt, fs, io, path::PathBuf};

use gpui::App;

use crate::theme::{
    AppearanceMode, ThemeActivationError, ThemeDocument, ThemeLoadError, activate_theme,
    parse_theme,
};

/// A validated theme file that can be explicitly reloaded into a GPUI app.
///
/// This type does not create a background watcher. Call [`reload`](Self::reload)
/// or [`reload_if_changed`](Self::reload_if_changed) when the app should check
/// the source file. The last successfully activated document remains available
/// if a later read, parse, or activation fails.
pub struct ThemeFile {
    path: PathBuf,
    document: ThemeDocument,
    activated_source: Option<String>,
    activated_mode: Option<AppearanceMode>,
}

impl ThemeFile {
    /// Read and validate a theme file without changing the app's active theme.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, ThemeFileError> {
        let path = path.into();
        let source = fs::read_to_string(&path)?;
        let document = parse_theme(&source)?;
        Ok(Self {
            path,
            document,
            activated_source: None,
            activated_mode: None,
        })
    }

    /// Return the most recently opened or successfully activated document.
    pub fn theme(&self) -> &ThemeDocument {
        &self.document
    }

    /// Reload and force-activate the file, even if its content is unchanged.
    pub fn reload(
        &mut self,
        mode: Option<AppearanceMode>,
        cx: &mut App,
    ) -> Result<bool, ThemeFileError> {
        self.reload_file(mode, true, |document| activate_theme(document, mode, cx))
    }

    /// Check the file and activate it only when its content or requested
    /// appearance mode has changed.
    ///
    /// Content is compared after reading the file, so edits with the same size
    /// and timestamp resolution are still detected.
    pub fn reload_if_changed(
        &mut self,
        mode: Option<AppearanceMode>,
        cx: &mut App,
    ) -> Result<bool, ThemeFileError> {
        self.reload_file(mode, false, |document| activate_theme(document, mode, cx))
    }

    fn reload_file(
        &mut self,
        mode: Option<AppearanceMode>,
        force: bool,
        activate: impl FnOnce(&ThemeDocument) -> Result<(), ThemeActivationError>,
    ) -> Result<bool, ThemeFileError> {
        let source = fs::read_to_string(&self.path)?;
        self.reload_source(source, mode, force, activate)
    }

    fn reload_source(
        &mut self,
        source: String,
        mode: Option<AppearanceMode>,
        force: bool,
        activate: impl FnOnce(&ThemeDocument) -> Result<(), ThemeActivationError>,
    ) -> Result<bool, ThemeFileError> {
        if !force
            && self.activated_source.as_deref() == Some(source.as_str())
            && self.activated_mode == mode
        {
            return Ok(false);
        }

        let candidate = parse_theme(&source)?;
        activate(&candidate)?;
        self.document = candidate;
        self.activated_source = Some(source);
        self.activated_mode = mode;
        Ok(true)
    }
}

#[derive(Debug)]
pub enum ThemeFileError {
    Io(io::Error),
    Load(ThemeLoadError),
    Activation(ThemeActivationError),
}

impl fmt::Display for ThemeFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not read theme file: {error}"),
            Self::Load(error) => write!(formatter, "could not load theme file: {error}"),
            Self::Activation(error) => write!(formatter, "could not activate theme file: {error}"),
        }
    }
}

impl Error for ThemeFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Load(error) => Some(error),
            Self::Activation(error) => Some(error),
        }
    }
}

impl From<io::Error> for ThemeFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ThemeLoadError> for ThemeFileError {
    fn from(error: ThemeLoadError) -> Self {
        Self::Load(error)
    }
}

impl From<ThemeActivationError> for ThemeFileError {
    fn from(error: ThemeActivationError) -> Self {
        Self::Activation(error)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::theme::ThemeResolveError;

    static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

    struct TempTheme(PathBuf);

    impl TempTheme {
        fn new(source: &str) -> Self {
            let directory = loop {
                let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir()
                    .join(format!("rustui-theme-reload-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => break path,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create unique temporary theme directory: {error}"),
                }
            };
            let path = directory.join("theme.json");
            fs::write(&path, source).expect("write temporary theme");
            Self(path)
        }

        fn write(&self, source: &str) {
            fs::write(&self.0, source).expect("update temporary theme");
        }
    }

    impl Drop for TempTheme {
        fn drop(&mut self) {
            if let Some(directory) = self.0.parent() {
                let _ = fs::remove_dir_all(directory);
            }
        }
    }

    fn missing_path(theme: &TempTheme) -> PathBuf {
        theme.0.with_file_name("missing.json")
    }

    fn valid_source() -> String {
        include_str!("../../../../themes/macos27.json").to_owned()
    }

    fn activate_ok(_: &ThemeDocument) -> Result<(), ThemeActivationError> {
        Ok(())
    }

    #[gpui::test]
    fn forced_reload_restores_file_after_another_theme_activation(cx: &mut gpui::TestAppContext) {
        let source = valid_source();
        let file = TempTheme::new(&source);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        cx.update(|app| {
            gpui_component::init(app);
            assert!(
                theme
                    .reload_if_changed(Some(AppearanceMode::Light), app)
                    .unwrap()
            );
            let document = parse_theme(&source).unwrap();
            activate_theme(&document, Some(AppearanceMode::Dark), app).unwrap();
            let revision = crate::theme::registry::active_theme(app)
                .unwrap()
                .revision();
            assert_eq!(
                crate::theme::registry::active_theme(app).unwrap().mode,
                AppearanceMode::Dark
            );
            assert!(theme.reload(Some(AppearanceMode::Light), app).unwrap());
            let active = crate::theme::registry::active_theme(app).unwrap();
            assert_eq!(active.mode, AppearanceMode::Light);
            assert_eq!(active.revision(), revision + 1);
            file.write("not json");
            assert!(theme.reload(Some(AppearanceMode::Dark), app).is_err());
            let active = crate::theme::registry::active_theme(app).unwrap();
            assert_eq!(active.mode, AppearanceMode::Light);
            assert_eq!(active.revision(), revision + 1);
        });
    }

    #[gpui::test]
    fn file_reload_recovers_after_invalid_update_in_app(cx: &mut gpui::TestAppContext) {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        cx.update(|app| {
            gpui_component::init(app);
            assert!(
                theme
                    .reload_if_changed(Some(AppearanceMode::Light), app)
                    .unwrap()
            );
            let revision = crate::theme::registry::active_theme(app)
                .unwrap()
                .revision();

            file.write("not json");
            assert!(
                theme
                    .reload_if_changed(Some(AppearanceMode::Light), app)
                    .is_err()
            );
            assert_eq!(
                crate::theme::registry::active_theme(app)
                    .unwrap()
                    .revision(),
                revision
            );
            assert_eq!(theme.theme(), &parse_theme(&initial).unwrap());

            file.write(&initial);
            assert!(
                !theme
                    .reload_if_changed(Some(AppearanceMode::Light), app)
                    .unwrap()
            );
            assert_eq!(
                crate::theme::registry::active_theme(app)
                    .unwrap()
                    .revision(),
                revision
            );

            file.write(&format!("{initial}\n"));
            assert!(
                theme
                    .reload_if_changed(Some(AppearanceMode::Light), app)
                    .unwrap()
            );
            assert_eq!(
                crate::theme::registry::active_theme(app)
                    .unwrap()
                    .revision(),
                revision + 1
            );
        });
    }

    #[test]
    fn reload_activates_changed_content_and_skips_identical_content() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");

        assert!(
            theme
                .reload_source(initial.clone(), None, true, activate_ok)
                .expect("initial activation")
        );
        assert!(
            !theme
                .reload_source(initial.clone(), None, false, activate_ok)
                .expect("unchanged reload")
        );

        let changed = format!("{initial}\n");
        let activations = AtomicUsize::new(0);
        assert!(
            theme
                .reload_source(changed, None, false, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("changed reload")
        );
        assert_eq!(activations.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn reload_file_reads_changes_and_first_check_activates() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        let activations = AtomicUsize::new(0);
        let activate = |_: &ThemeDocument| {
            activations.fetch_add(1, Ordering::Relaxed);
            Ok(())
        };

        assert!(
            theme
                .reload_file(None, false, activate)
                .expect("first file check activates")
        );
        assert!(
            !theme
                .reload_file(None, false, activate)
                .expect("unchanged file check skips")
        );

        let changed = format!("{initial}\n");
        file.write(&changed);
        assert!(
            theme
                .reload_file(None, false, activate)
                .expect("changed file activates")
        );
        assert!(
            !theme
                .reload_file(None, false, activate)
                .expect("second unchanged check skips")
        );
        assert_eq!(activations.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn changed_mode_and_forced_reload_reactivate_identical_content() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        let activations = AtomicUsize::new(0);

        assert!(
            theme
                .reload_source(initial.clone(), Some(AppearanceMode::Light), false, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("first activation")
        );
        assert!(
            theme
                .reload_source(initial.clone(), Some(AppearanceMode::Dark), false, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("mode change")
        );
        assert!(
            theme
                .reload_source(initial, Some(AppearanceMode::Dark), true, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("forced reload")
        );
        assert_eq!(activations.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn failed_reload_keeps_last_good_theme_and_can_recover() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        theme
            .reload_source(initial.clone(), None, true, activate_ok)
            .expect("initial activation");

        file.write("not json");
        let invalid = fs::read_to_string(&file.0).expect("read invalid fixture");
        assert!(
            theme
                .reload_source(invalid, None, false, activate_ok)
                .is_err()
        );
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );

        let ambiguous = initial.replacen(
            "\"background\": \"surface\",",
            "\"background\": \"ignored\",\n    \"background\": \"surface\",",
            1,
        );
        assert_ne!(ambiguous, initial);
        assert!(
            theme
                .reload_source(ambiguous, None, false, activate_ok)
                .is_err()
        );
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );

        file.write(&initial);
        let repaired = fs::read_to_string(&file.0).expect("read repaired fixture");
        assert!(
            !theme
                .reload_source(repaired, None, false, activate_ok)
                .expect("repaired content matches last good")
        );
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );
    }

    #[test]
    fn missing_file_does_not_replace_last_good_document_and_recovers() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        theme
            .reload_source(initial.clone(), None, true, activate_ok)
            .expect("initial activation");
        fs::remove_file(&file.0).expect("remove theme file");

        assert!(matches!(
            theme.reload_file(None, false, activate_ok),
            Err(ThemeFileError::Io(_))
        ));
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );

        file.write(&initial);
        assert!(
            !theme
                .reload_file(None, false, activate_ok)
                .expect("recovered unchanged source")
        );
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );
    }

    #[test]
    fn activation_failure_does_not_commit_candidate() {
        let initial = valid_source();
        let file = TempTheme::new(&initial);
        let mut theme = ThemeFile::open(&file.0).expect("open theme");
        theme
            .reload_source(initial.clone(), None, true, activate_ok)
            .expect("initial activation");

        let changed = format!("{initial}\n");
        let result = theme.reload_source(changed.clone(), None, false, |_| {
            Err(ThemeActivationError::Resolve(ThemeResolveError::Invalid(
                "simulated activation failure".into(),
            )))
        });
        assert!(matches!(result, Err(ThemeFileError::Activation(_))));
        assert_eq!(
            theme.theme(),
            &parse_theme(&initial).expect("original theme")
        );
        assert_eq!(theme.activated_source.as_deref(), Some(initial.as_str()));
        assert_eq!(theme.activated_mode, None);

        let activations = AtomicUsize::new(0);
        assert!(
            theme
                .reload_source(changed.clone(), None, false, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("retry identical candidate after activation failure")
        );
        assert_eq!(theme.activated_source.as_deref(), Some(changed.as_str()));
        assert_eq!(theme.activated_mode, None);
        assert_eq!(activations.load(Ordering::Relaxed), 1);

        assert!(
            !theme
                .reload_source(changed.clone(), None, false, |_| {
                    activations.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .expect("unchanged candidate skips activation")
        );
        assert_eq!(activations.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn open_reports_missing_file_and_invalid_file_can_be_repaired() {
        let owner = TempTheme::new("owned fixture");
        let path = missing_path(&owner);
        assert!(matches!(ThemeFile::open(&path), Err(ThemeFileError::Io(_))));

        let file = TempTheme::new("invalid");
        assert!(matches!(
            ThemeFile::open(&file.0),
            Err(ThemeFileError::Load(_))
        ));
        let valid = valid_source();
        file.write(&valid);
        assert!(ThemeFile::open(&file.0).is_ok());
    }
}
