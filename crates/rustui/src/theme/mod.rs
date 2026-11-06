//! Versioned platform themes and adapters.

pub(crate) mod invalidation;
pub mod loader;
pub mod registry;
pub mod resolver;
pub mod schema;
pub mod tokens;
pub mod watcher;

pub use crate::adapters::themes::{ThemeApplyError, apply_profile};
pub use loader::{ThemeLoadError, ThemeSerializeError, load_theme, parse_theme, serialize_theme};
pub use registry::{
    ActiveTheme, ThemeActivationError, activate_theme, assess_theme_coverage, theme_coverage,
};
pub use resolver::{
    ComponentStyleCapability, ResolvedButtonTheme, ResolvedComponentStyle, ThemeCoverageReport,
    ThemeRecordDisposition, ThemeResolveError, UncoveredComponentRule, resolve_button,
    resolve_button_with_selector, resolve_component_style, resolve_component_style_with,
};
pub use schema::*;
pub use watcher::{ThemeFile, ThemeFileError};
