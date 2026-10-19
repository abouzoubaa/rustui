//! Locale, catalog, fallback, and validation support for RustUI.

pub mod catalog;
pub mod direction;
pub mod fallback;
pub mod formatting;
pub mod locale;
pub mod message;
pub mod validation;

pub use catalog::{Catalog, CatalogError, Catalogs};
pub use direction::{BidiIsolation, LayoutDirection};
pub use locale::{Locale, LocaleError};
pub use message::{MessageId, MessageIdError};
pub use validation::{CatalogValidation, validate_catalogs};
