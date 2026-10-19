use std::collections::BTreeSet;

const CATALOGS: &[&str] = &[
    include_str!("../../../locales/en/rustui.ftl"),
    include_str!("../../../locales/fr/rustui.ftl"),
    include_str!("../../../locales/ar/rustui.ftl"),
];

fn message_ids(catalog: &str) -> BTreeSet<&str> {
    catalog
        .lines()
        .filter_map(|line| line.split_once(" = ").map(|(id, _)| id))
        .collect()
}

#[test]
fn framework_catalogs_define_the_same_message_ids() {
    let expected = message_ids(CATALOGS[0]);
    assert!(!expected.is_empty());

    for catalog in &CATALOGS[1..] {
        assert_eq!(message_ids(catalog), expected);
    }
}
