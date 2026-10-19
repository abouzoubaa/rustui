//! Logical layout direction, distinct from Unicode bidirectional text shaping.

use crate::Locale;

/// Unicode bidi isolation mode for dynamically inserted text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BidiIsolation {
    /// Let the Unicode bidi algorithm choose direction from the first strong character.
    FirstStrong,
    /// Force left-to-right direction inside the isolate.
    LeftToRight,
    /// Force right-to-left direction inside the isolate.
    RightToLeft,
}

impl BidiIsolation {
    /// Wraps text with this isolate and a matching PDI.
    pub fn wrap(self, text: &str) -> String {
        let opener = match self {
            Self::FirstStrong => '\u{2068}', // FSI
            Self::LeftToRight => '\u{2066}', // LRI
            Self::RightToLeft => '\u{2067}', // RLI
        };
        let mut isolated = String::with_capacity(text.len() + opener.len_utf8() + 3);
        isolated.push(opener);
        isolated.push_str(text);
        isolated.push('\u{2069}'); // PDI
        isolated
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

impl LayoutDirection {
    pub fn for_locale(locale: &Locale) -> Self {
        if let Some(script) = explicit_script(locale) {
            return if is_right_to_left_script(script) {
                Self::RightToLeft
            } else {
                Self::LeftToRight
            };
        }

        let has_rtl_language = matches!(
            locale.primary_language(),
            "ar" | "dv" | "fa" | "he" | "iw" | "ps" | "sd" | "ug" | "ur" | "yi"
        );

        if has_rtl_language {
            Self::RightToLeft
        } else {
            Self::LeftToRight
        }
    }

    pub fn is_right_to_left(self) -> bool {
        matches!(self, Self::RightToLeft)
    }

    /// Wraps dynamic text in Unicode bidi isolate controls.
    ///
    /// The returned controls isolate this string from surrounding text; they do
    /// not translate, shape, or otherwise alter its contents.
    pub fn isolate_text(self, text: &str) -> String {
        let isolation = if self.is_right_to_left() {
            BidiIsolation::RightToLeft
        } else {
            BidiIsolation::LeftToRight
        };
        isolation.wrap(text)
    }
}

/// Returns a script subtag only when it appears in the BCP 47 script position.
/// This parser intentionally follows Locale's permissive subtag validation and
/// does not add validation rules for extlangs, regions, variants, or extensions.
fn explicit_script(locale: &Locale) -> Option<&str> {
    let mut subtags = locale.subtags();
    let language = subtags.next()?;
    let mut extlangs_left = if (2..=3).contains(&language.len()) {
        3
    } else {
        0
    };

    for subtag in subtags {
        if subtag.len() == 4 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return Some(subtag);
        }

        if extlangs_left > 0 && subtag.len() == 3 && subtag.bytes().all(|b| b.is_ascii_alphabetic())
        {
            extlangs_left -= 1;
            continue;
        }

        // A region or any later BCP 47 subtag means the script position passed.
        if (subtag.len() == 2 && subtag.bytes().all(|b| b.is_ascii_alphabetic()))
            || (subtag.len() == 3 && subtag.bytes().all(|b| b.is_ascii_digit()))
            || subtag.len() == 1
        {
            return None;
        }

        // The current Locale accepts nonstandard subtag sequences. Stop rather
        // than treating a later variant as a script.
        return None;
    }
    None
}

fn is_right_to_left_script(script: &str) -> bool {
    matches!(
        script,
        "Adlm" | "Arab" | "Hebr" | "Mand" | "Nkoo" | "Rohg" | "Samr" | "Syrc" | "Thaa"
    )
}

#[cfg(test)]
mod tests {
    use super::{BidiIsolation, LayoutDirection};
    use crate::Locale;

    #[test]
    fn isolates_unicode_and_empty_text_with_each_control() {
        let ltr = LayoutDirection::LeftToRight;
        assert_eq!(
            BidiIsolation::FirstStrong.wrap("مرحبا"),
            "\u{2068}مرحبا\u{2069}"
        );
        assert_eq!(ltr.isolate_text("שלום"), "\u{2066}שלום\u{2069}");
        assert_eq!(
            LayoutDirection::RightToLeft.isolate_text(""),
            "\u{2067}\u{2069}"
        );
    }

    #[test]
    fn explicit_script_overrides_language_direction() {
        let direction = |tag| LayoutDirection::for_locale(&Locale::parse(tag).unwrap());
        assert_eq!(direction("ar"), LayoutDirection::RightToLeft);
        assert_eq!(direction("ar-Latn"), LayoutDirection::LeftToRight);
        assert_eq!(direction("en-Arab"), LayoutDirection::RightToLeft);
        assert_eq!(direction("en-Hebr"), LayoutDirection::RightToLeft);
    }

    #[test]
    fn a_later_variant_does_not_override_language_direction_as_a_script() {
        let locale = Locale::parse("ar-1901-Latn").unwrap();
        assert_eq!(
            LayoutDirection::for_locale(&locale),
            LayoutDirection::RightToLeft
        );
    }
}
