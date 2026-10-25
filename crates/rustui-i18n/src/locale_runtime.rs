//! ICU4X formatting with RustUI-owned civil values and explicit field sets.
//!
//! ICU4X 2.4 compiled formatting data is generated from CLDR 49 / Unicode 18.
//! This data contract is independent of Fluent's plural-selector data.
use std::{error::Error, fmt, str::FromStr};

use icu_datetime::{
    DateTimeFormatter, NoCalendarFormatter, fieldsets,
    input::{Date, DateTime, Time},
};
use icu_decimal::{DecimalFormatter, input::Decimal};
use icu_locale_core::Locale as IcuLocale;
use writeable::Writeable;

use crate::Locale;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleFormatError(pub(crate) String);
impl fmt::Display for LocaleFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for LocaleFormatError {}
fn error(value: impl fmt::Display) -> LocaleFormatError {
    LocaleFormatError(value.to_string())
}

/// An ISO civil date, converted to the locale's selected calendar for display.
/// No time zone or timestamp interpretation is performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilDateTime {
    pub date: CivilDate,
    pub time: CivilTime,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateLength {
    Short,
    #[default]
    Medium,
    Long,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFields {
    #[default]
    HourMinute,
    HourMinuteSecond,
}

/// Reusable number formatter. Unicode `u-nu` preferences are passed to ICU.
/// Currency, percent, units, compact notation and rounding options are not
/// part of this API; they must not be silently treated as decimal formatting.
pub struct LocaleFormatter {
    locale: IcuLocale,
    decimal: DecimalFormatter,
}
impl LocaleFormatter {
    pub fn try_new(locale: &Locale) -> Result<Self, LocaleFormatError> {
        let locale = IcuLocale::from_str(locale.as_str()).map_err(error)?;
        icu_decimal::DecimalFormatterPreferences::from_locale_strict(&locale)
            .map_err(|_| LocaleFormatError("unsupported ICU number preference".to_owned()))?;
        icu_datetime::DateTimeFormatterPreferences::from_locale_strict(&locale)
            .map_err(|_| LocaleFormatError("unsupported ICU date/time preference".to_owned()))?;
        let decimal =
            DecimalFormatter::try_new(locale.clone().into(), Default::default()).map_err(error)?;
        Ok(Self { locale, decimal })
    }
    /// Exact decimal input, including trailing zeroes. No binary float conversion.
    pub fn format_decimal(&self, value: &str) -> Result<String, LocaleFormatError> {
        let value = Decimal::from_str(value).map_err(error)?;
        Ok(self.decimal.format(&value).write_to_string().into_owned())
    }
    pub fn format_integer(&self, value: i64) -> String {
        self.decimal
            .format(&Decimal::from(value))
            .write_to_string()
            .into_owned()
    }
    /// Formats year, month and day using locale calendar (`u-ca`) preferences.
    pub fn format_date(
        &self,
        value: CivilDate,
        length: DateLength,
    ) -> Result<String, LocaleFormatError> {
        let date = Date::try_new_iso(value.year, value.month, value.day).map_err(error)?;
        let fields = match length {
            DateLength::Short => fieldsets::YMD::short(),
            DateLength::Medium => fieldsets::YMD::medium(),
            DateLength::Long => fieldsets::YMD::long(),
        };
        let formatter =
            DateTimeFormatter::try_new(self.locale.clone().into(), fields).map_err(error)?;
        Ok(formatter.format(&date).write_to_string().into_owned())
    }
    /// Formats local wall-clock fields. This never performs zone conversion.
    pub fn format_time(
        &self,
        value: CivilTime,
        fields: TimeFields,
    ) -> Result<String, LocaleFormatError> {
        let time = Time::try_new(value.hour, value.minute, value.second, 0).map_err(error)?;
        match fields {
            TimeFields::HourMinute => {
                let formatter =
                    NoCalendarFormatter::try_new(self.locale.clone().into(), fieldsets::T::hm())
                        .map_err(error)?;
                Ok(formatter.format(&time).write_to_string().into_owned())
            }
            TimeFields::HourMinuteSecond => {
                let formatter =
                    NoCalendarFormatter::try_new(self.locale.clone().into(), fieldsets::T::hms())
                        .map_err(error)?;
                Ok(formatter.format(&time).write_to_string().into_owned())
            }
        }
    }
    pub fn format_datetime(
        &self,
        value: CivilDateTime,
        length: DateLength,
        fields: TimeFields,
    ) -> Result<String, LocaleFormatError> {
        let date =
            Date::try_new_iso(value.date.year, value.date.month, value.date.day).map_err(error)?;
        let time = Time::try_new(value.time.hour, value.time.minute, value.time.second, 0)
            .map_err(error)?;
        let value = DateTime { date, time };
        let date_fields = match length {
            DateLength::Short => fieldsets::YMD::short(),
            DateLength::Medium => fieldsets::YMD::medium(),
            DateLength::Long => fieldsets::YMD::long(),
        };
        match fields {
            TimeFields::HourMinute => {
                let formatter = DateTimeFormatter::try_new(
                    self.locale.clone().into(),
                    date_fields.with_time_hm(),
                )
                .map_err(error)?;
                Ok(formatter.format(&value).write_to_string().into_owned())
            }
            TimeFields::HourMinuteSecond => {
                let formatter = DateTimeFormatter::try_new(
                    self.locale.clone().into(),
                    date_fields.with_time_hms(),
                )
                .map_err(error)?;
                Ok(formatter.format(&value).write_to_string().into_owned())
            }
        }
    }
}
