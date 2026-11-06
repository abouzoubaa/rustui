//! Locale, catalog, fallback, and validation support for RustUI.
//!
//! Legacy catalogs keep their original placeholder grammar. Use explicit Fluent
//! resources for terms, references, attributes and Fluent selectors:
//!
//! ```
//! use rustui_i18n::{Catalog, Catalogs, Locale, LocaleFormatter, MessageArguments};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let locale = Locale::parse("en-US")?;
//! let mut catalogs = Catalogs::new(locale.clone());
//! catalogs.insert(Catalog::parse_fluent(locale.clone(),
//!     "-brand = RustUI\nwelcome = Welcome to { -brand }, { $name }!"
//! )?)?;
//! let message = catalogs.format(&locale, "welcome",
//!     &MessageArguments::new().with("name", "Ada"))?;
//! assert!(message.contains("RustUI"));
//! assert!(message.contains("Ada"));
//! assert_eq!(LocaleFormatter::try_new(&locale)?.format_decimal("1234.50")?, "1,234.50");
//! # Ok(())
//! # }
//! ```

pub mod catalog;
pub mod direction;
pub mod fallback;
pub mod fluent;
pub mod formatting;
pub mod locale;
pub mod locale_runtime;
pub mod message;
pub mod validation;

pub use catalog::{Catalog, CatalogError, Catalogs};
pub use direction::{BidiIsolation, LayoutDirection};
pub use formatting::{
    InterpolationError, MessageArgument, MessageArguments, format_decimal, format_integer,
    interpolate, interpolate_with_locale,
};
pub use locale::{Locale, LocaleError};
pub use message::{MessageId, MessageIdError};
pub use validation::{CatalogValidation, validate_catalogs};

pub use fluent::FluentFormatError;
pub use locale_runtime::{
    CivilDate, CivilDateTime, CivilTime, DateLength, LocaleFormatError, LocaleFormatter,
    TimeFields, Weekday,
};
