use rustui::theme::{AppearanceMode, parse_theme};

#[test]
fn profile_colors_use_default_when_mode_is_unspecified_and_honor_explicit_modes() {
    let document = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("parse existing macOS 27 theme");
    let mut profile = document.platform_snapshot.clone();
    profile.appearance.default = AppearanceMode::Dark;

    assert_eq!(profile.colors(None), &profile.appearance.dark);
    assert_eq!(
        profile.colors(Some(AppearanceMode::Light)),
        &profile.appearance.light
    );
    assert_eq!(
        profile.colors(Some(AppearanceMode::Dark)),
        &profile.appearance.dark
    );
}
