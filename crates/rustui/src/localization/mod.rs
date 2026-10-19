//! Locale coordination shared by the RustUI facade and GPUI adapters.
//!
//! Application message catalogs and locale policy belong to `rustui-i18n`.
//! GPUI Components locale updates and view invalidation are integrated here.

pub mod gpui_adapter;
pub mod invalidation;

use gpui::{App, Global};

use crate::runtime::AppAction;
pub use rustui_i18n::{
    Catalog, CatalogError, CatalogValidation, Catalogs, LayoutDirection, Locale, LocaleError,
    MessageId, MessageIdError, validate_catalogs,
};

#[derive(Clone, Debug)]
pub struct ActiveLocalization {
    locale: Locale,
    direction: LayoutDirection,
    catalogs: Catalogs,
    generation: u64,
}

impl ActiveLocalization {
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    pub fn direction(&self) -> LayoutDirection {
        self.direction
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn message<'a>(&'a self, identifier: &'a str) -> &'a str {
        self.catalogs.resolve(&self.locale, identifier)
    }
}

impl Global for ActiveLocalization {}

pub fn active_localization(cx: &App) -> Option<&ActiveLocalization> {
    cx.try_global::<ActiveLocalization>()
}

pub fn activate_localization(catalogs: Catalogs, locale: Locale, cx: &mut App) {
    let generation = cx
        .try_global::<ActiveLocalization>()
        .map_or(0, |active| active.generation.saturating_add(1));
    gpui_adapter::set_locale(&locale);
    cx.set_global(ActiveLocalization {
        direction: LayoutDirection::for_locale(&locale),
        locale,
        catalogs,
        generation,
    });
}

pub fn set_locale(locale: Locale, cx: &mut App) {
    let catalogs = cx
        .try_global::<ActiveLocalization>()
        .map(|active| active.catalogs.clone())
        .unwrap_or_else(|| Catalogs::new(locale.clone()));
    activate_localization(catalogs, locale, cx);
}

pub fn set_locale_action(locale: Locale) -> AppAction {
    AppAction::new(move |cx| set_locale(locale.clone(), cx))
}
