const ARABIC_LOCALE: &str = include_str!("../../../locales/ar/locale.json");
const ARABIC_CATALOG: &str = include_str!("../../../locales/ar/rustui.ftl");

#[test]
fn arabic_locale_metadata_and_catalog_are_rtl_ready() {
    let metadata: serde_json::Value =
        serde_json::from_str(ARABIC_LOCALE).expect("valid Arabic locale metadata");
    assert_eq!(metadata["direction"], "rtl");
    assert!(!ARABIC_CATALOG.is_ascii());
}
