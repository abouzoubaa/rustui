//! Cross-catalog validation against the configured fallback catalog.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Catalogs, Locale, MessageId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogValidation {
    pub fallback_catalog_missing: bool,
    pub missing: BTreeMap<Locale, Vec<MessageId>>,
    pub extra: BTreeMap<Locale, Vec<MessageId>>,
}

impl CatalogValidation {
    pub fn is_valid(&self) -> bool {
        !self.fallback_catalog_missing && self.missing.is_empty() && self.extra.is_empty()
    }
}

pub fn validate_catalogs(catalogs: &Catalogs) -> CatalogValidation {
    let Some(reference) = catalogs.get(catalogs.fallback()) else {
        return CatalogValidation {
            fallback_catalog_missing: true,
            ..CatalogValidation::default()
        };
    };
    let expected = reference.message_ids().cloned().collect::<BTreeSet<_>>();
    let mut validation = CatalogValidation::default();

    for (locale, catalog) in catalogs.iter() {
        let actual = catalog.message_ids().cloned().collect::<BTreeSet<_>>();
        let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
        let extra = actual.difference(&expected).cloned().collect::<Vec<_>>();
        if !missing.is_empty() {
            validation.missing.insert(locale.clone(), missing);
        }
        if !extra.is_empty() {
            validation.extra.insert(locale.clone(), extra);
        }
    }

    validation
}
