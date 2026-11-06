//! Locale coordination shared by the RustUI facade and GPUI adapters.
//!
//! Application message catalogs and locale policy belong to `rustui-i18n`.
//! GPUI Components locale updates and view invalidation are integrated here.

pub mod gpui_adapter;
pub mod invalidation;

use gpui::{App, Global};

use crate::runtime::AppAction;
pub use rustui_i18n::{
    Catalog, CatalogError, CatalogValidation, Catalogs, InterpolationError, LayoutDirection,
    Locale, LocaleError, MessageArgument, MessageArguments, MessageId, MessageIdError,
    format_integer, interpolate, interpolate_with_locale, validate_catalogs,
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

    /// Resolves a catalog message for the active locale, then interpolates its named arguments.
    pub fn message_with_arguments(
        &self,
        identifier: &str,
        arguments: &MessageArguments,
    ) -> Result<String, InterpolationError> {
        interpolate_with_locale(self.message(identifier), arguments, &self.locale)
    }
}

pub(crate) fn resolve_message_with_arguments(
    localization: Option<&ActiveLocalization>,
    identifier: &str,
    arguments: &MessageArguments,
) -> Result<String, InterpolationError> {
    match localization {
        Some(active) => active.message_with_arguments(identifier, arguments),
        None => interpolate(identifier, arguments),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_localization(locale_id: &str, catalogs: Catalogs) -> ActiveLocalization {
        let locale = Locale::parse(locale_id).unwrap();
        ActiveLocalization {
            direction: LayoutDirection::for_locale(&locale),
            locale,
            catalogs,
            generation: 0,
        }
    }

    fn catalog(locale_id: &str, source: &str) -> Catalog {
        Catalog::parse(Locale::parse(locale_id).unwrap(), source).unwrap()
    }

    #[test]
    fn message_with_arguments_uses_active_locale_catalog_and_fallback() {
        let fallback = Locale::parse("en").unwrap();
        let mut catalogs = Catalogs::new(fallback);
        catalogs
            .insert(catalog(
                "en",
                "greeting = Hello, {name}!\nfallback-only = From fallback, {name}.",
            ))
            .unwrap();
        catalogs
            .insert(catalog("fr", "greeting = Bonjour, {name}!"))
            .unwrap();
        let active = active_localization("fr-CA", catalogs);
        let arguments = MessageArguments::new().with("name", "Ada");

        assert_eq!(
            active.message_with_arguments("greeting", &arguments),
            Ok("Bonjour, Ada!".to_owned())
        );
        assert_eq!(
            active.message_with_arguments("fallback-only", &arguments),
            Ok("From fallback, Ada.".to_owned())
        );
    }

    #[test]
    fn message_with_arguments_evaluates_locale_plural_and_select_expressions() {
        let fallback = Locale::parse("en").unwrap();
        let mut catalogs = Catalogs::new(fallback);
        catalogs
            .insert(catalog(
                "en",
                "items = {count, plural, one {one item} other {{count} items}}\nlabel = {gender, select, female {She} other {They}}",
            ))
            .unwrap();
        catalogs
            .insert(catalog(
                "fr",
                "items = {count, plural, one {{count} article} other {{count} articles}}",
            ))
            .unwrap();
        let active = active_localization("fr", catalogs);
        assert_eq!(
            active.message_with_arguments("items", &MessageArguments::new().with("count", 0_i64),),
            Ok("0 article".to_owned())
        );
        assert_eq!(
            active.message_with_arguments(
                "label",
                &MessageArguments::new().with("gender", "female"),
            ),
            Ok("She".to_owned())
        );
    }

    #[test]
    fn message_with_arguments_returns_typed_interpolation_errors() {
        let catalogs = Catalogs::new(Locale::parse("en").unwrap());
        let active = active_localization("en", catalogs);

        assert_eq!(
            active.message_with_arguments("Hello, {name}", &MessageArguments::new()),
            Err(InterpolationError::MissingArgument("name".to_owned()))
        );
    }

    #[test]
    fn message_with_arguments_formats_integer_for_active_locale() {
        let catalogs = Catalogs::new(Locale::parse("en").unwrap());
        let active = active_localization("fr", catalogs);
        let arguments = MessageArguments::new().with("count", 12_345_i64);

        assert_eq!(
            active.message_with_arguments("{count} fichiers", &arguments),
            Ok("12\u{202f}345 fichiers".to_owned())
        );
    }

    #[test]
    fn message_with_arguments_formats_decimal_for_active_locale() {
        let catalogs = Catalogs::new(Locale::parse("en").unwrap());
        let active = active_localization("fr", catalogs);
        let arguments = MessageArguments::new().with("amount", 1_234.5_f64);

        assert_eq!(
            active.message_with_arguments("{amount} euros", &arguments),
            Ok("1\u{202f}234,5 euros".to_owned())
        );
    }

    #[test]
    fn message_with_arguments_selects_decimal_plural_for_active_locale() {
        let catalogs = Catalogs::new(Locale::parse("en").unwrap());
        let active = active_localization("fr", catalogs);
        let arguments = MessageArguments::new().with("count", 1.5_f64);

        assert_eq!(
            active.message_with_arguments(
                "{count, plural, one {article} other {articles}}",
                &arguments
            ),
            Ok("article".to_owned())
        );
    }

    #[test]
    fn runtime_text_resolution_uses_active_locale_and_catalog_fallback() {
        let fallback = Locale::parse("en").unwrap();
        let mut catalogs = Catalogs::new(fallback);
        catalogs
            .insert(catalog(
                "en",
                "greeting = Hello, {name}!\nfallback-only = From fallback, {name}.",
            ))
            .unwrap();
        catalogs
            .insert(catalog("fr", "greeting = Bonjour, {name}!"))
            .unwrap();
        let active = active_localization("fr-CA", catalogs);
        let arguments = MessageArguments::new().with("name", "Ada");

        assert_eq!(
            resolve_message_with_arguments(Some(&active), "greeting", &arguments),
            Ok("Bonjour, Ada!".to_owned())
        );
        assert_eq!(
            resolve_message_with_arguments(Some(&active), "fallback-only", &arguments),
            Ok("From fallback, Ada.".to_owned())
        );
        assert_eq!(
            resolve_message_with_arguments(Some(&active), "greeting", &MessageArguments::new()),
            Err(InterpolationError::MissingArgument("name".to_owned()))
        );
    }

    #[test]
    fn runtime_text_without_active_locale_interpolates_identifier() {
        let arguments = MessageArguments::new().with("name", "Ada");
        assert_eq!(
            resolve_message_with_arguments(None, "Hello, {name}!", &arguments),
            Ok("Hello, Ada!".to_owned())
        );
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
