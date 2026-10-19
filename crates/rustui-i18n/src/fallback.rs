//! Catalog fallback is implemented by [`crate::Catalogs`].
//!
//! Lookup order is the exact locale, its primary language, the configured
//! fallback locale, its primary language, and finally the message identifier.
