const CATALOGS: &[&str] = &[
    include_str!("../../../locales/en/rustui.ftl"),
    include_str!("../../../locales/fr/rustui.ftl"),
    include_str!("../../../locales/ar/rustui.ftl"),
];

#[test]
fn each_supported_locale_has_framework_messages() {
    for catalog in CATALOGS {
        assert!(catalog.contains("rustui-action-cancel ="));
        assert!(catalog.contains("rustui-state-loading ="));
    }
}
