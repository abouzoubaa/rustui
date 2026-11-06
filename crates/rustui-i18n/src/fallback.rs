//! Catalog fallback is implemented by [`crate::Catalogs`].
//!
//! Lookup order is the exact locale, its extension-free base, its primary
//! language, the configured fallback locale, its extension-free base, its
//! primary language, and finally the message identifier.
