//! Typed access to existing appearance roles; values are resolved on demand.
use super::resolver::resolve_platform_color;
use super::{AppearanceMode, RgbaValue, ThemeProfile, ThemeResolveError, registry::ActiveTheme};

/// Color roles used by navigation surfaces and shared application chrome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeToken {
    SidebarBackground,
    SidebarForeground,
    SidebarSelectionBackground,
    ToolbarBackground,
    ToolbarForeground,
    TabBarBackground,
    TabBarSegmentedBackground,
    TabBarTransparentBackground,
    TabBarForeground,
    TabActiveBackground,
    TabActiveForeground,
    SurfaceBackground,
    PrimaryForeground,
    Border,
    Accent,
}

impl ThemeProfile {
    /// Resolve a typed color from the existing appearance profile.
    pub fn token(
        &self,
        mode: AppearanceMode,
        token: ThemeToken,
    ) -> Result<RgbaValue, ThemeResolveError> {
        let colors = self.colors(Some(mode));
        if token == ThemeToken::TabBarTransparentBackground {
            return Ok(RgbaValue {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            });
        }
        let (value, opacity) = match token {
            ThemeToken::SidebarBackground => {
                (&colors.sidebar_background, self.materials.sidebar_opacity)
            }
            ThemeToken::SidebarForeground
            | ThemeToken::ToolbarForeground
            | ThemeToken::TabActiveForeground
            | ThemeToken::PrimaryForeground => (&colors.primary_text, 1.0),
            ThemeToken::SidebarSelectionBackground | ThemeToken::TabActiveBackground => {
                (&colors.selected_row_background, 1.0)
            }
            ThemeToken::ToolbarBackground | ThemeToken::TabBarBackground => {
                (&colors.toolbar_background, self.materials.toolbar_opacity)
            }
            ThemeToken::TabBarSegmentedBackground => (&colors.hover_row_background, 1.0),
            ThemeToken::TabBarTransparentBackground => unreachable!(),
            ThemeToken::TabBarForeground => (&colors.secondary_text, 1.0),
            ThemeToken::SurfaceBackground => (&colors.content_background, 1.0),
            ThemeToken::Border => (&colors.border, 1.0),
            ThemeToken::Accent => (&colors.accent, 1.0),
        };
        let mut resolved = resolve_platform_color(value, "theme token")?;
        resolved.a *= opacity as f32;
        Ok(resolved)
    }
}

impl ActiveTheme {
    /// Resolve a token in the active appearance without storing another palette.
    pub fn token(&self, token: ThemeToken) -> Result<RgbaValue, ThemeResolveError> {
        self.profile.token(self.mode, token)
    }
}

/// Apply the resolved surface role colors and any supported body overrides.
pub(crate) fn apply_surface<T: gpui::Styled>(
    surface: T,
    theme: &ActiveTheme,
    component: &str,
    selector: &super::ButtonStyleSelector,
    background: ThemeToken,
    foreground: ThemeToken,
) -> T {
    let (background, foreground) = theme
        .surface_colors(component, selector, background, foreground)
        .unwrap_or_else(|error| panic!("{component} surface theme cannot resolve: {error}"));
    surface
        .bg(super::registry::hsla(background))
        .text_color(super::registry::hsla(foreground))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{
        activate_theme, assess_theme_coverage, load_theme, parse_theme, serialize_theme,
    };
    use gpui::Styled as _;
    use gpui_component::{
        sidebar::{Sidebar, SidebarMenuItem},
        tab::TabBar,
        toolbar::Toolbar,
    };

    #[test]
    fn typed_tokens_resolve_runtime_roles_and_theme_round_trips() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../themes/macos27.json");
        let document = load_theme(path).unwrap();
        assert_eq!(
            parse_theme(&serialize_theme(&document).unwrap()).unwrap(),
            document
        );
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let profile = &document.platform_snapshot;
            let colors = profile.colors(Some(mode));
            let cases = [
                (
                    ThemeToken::SidebarBackground,
                    &colors.sidebar_background,
                    profile.materials.sidebar_opacity,
                ),
                (ThemeToken::SidebarForeground, &colors.primary_text, 1.0),
                (
                    ThemeToken::SidebarSelectionBackground,
                    &colors.selected_row_background,
                    1.0,
                ),
                (
                    ThemeToken::ToolbarBackground,
                    &colors.toolbar_background,
                    profile.materials.toolbar_opacity,
                ),
                (ThemeToken::ToolbarForeground, &colors.primary_text, 1.0),
                (
                    ThemeToken::TabBarBackground,
                    &colors.toolbar_background,
                    profile.materials.toolbar_opacity,
                ),
                (
                    ThemeToken::TabBarSegmentedBackground,
                    &colors.hover_row_background,
                    1.0,
                ),
                (
                    ThemeToken::TabBarTransparentBackground,
                    &colors.content_background,
                    0.0,
                ),
                (ThemeToken::TabBarForeground, &colors.secondary_text, 1.0),
                (
                    ThemeToken::TabActiveBackground,
                    &colors.selected_row_background,
                    1.0,
                ),
                (ThemeToken::TabActiveForeground, &colors.primary_text, 1.0),
                (
                    ThemeToken::SurfaceBackground,
                    &colors.content_background,
                    1.0,
                ),
                (ThemeToken::PrimaryForeground, &colors.primary_text, 1.0),
                (ThemeToken::Border, &colors.border, 1.0),
                (ThemeToken::Accent, &colors.accent, 1.0),
            ];
            for (token, source, opacity) in cases {
                let mut expected = if token == ThemeToken::TabBarTransparentBackground {
                    RgbaValue {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }
                } else {
                    resolve_platform_color(source, "test").unwrap()
                };
                expected.a *= opacity as f32;
                assert_eq!(
                    profile.token(mode, token).unwrap(),
                    expected,
                    "{token:?} {mode:?}"
                );
            }
        }
    }

    #[gpui::test]
    fn navigation_surface_capabilities_resolve_and_reach_component_styles(
        cx: &mut gpui::TestAppContext,
    ) {
        let mut document = parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let template = document.components["checkbox"]["styles"][0].clone();
        for name in ["sidebar", "toolbar", "tab_bar"] {
            let mut rule = template.clone();
            rule["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .retain(|name, _| name == "background" || name == "foreground");
            document.components.get_mut(name).unwrap()["styles"] = serde_json::json!([rule]);
        }
        let coverage = assess_theme_coverage(&document);
        for name in ["sidebar", "toolbar", "tab_bar"] {
            assert!(
                coverage
                    .renderer_capabilities
                    .iter()
                    .any(|c| c.component == name
                        && c.part == "body"
                        && c.properties == ["background", "foreground"])
            );
            assert!(!coverage.uncovered_rules.iter().any(|r| r.component == name));
        }
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&document, Some(AppearanceMode::Dark), app).unwrap();
            let theme = super::super::registry::active_theme(app).unwrap();
            let selector = crate::components::theme::selector_for_mode(AppearanceMode::Dark);
            fn check<T: gpui::Styled>(mut element: T, expected: (RgbaValue, RgbaValue)) {
                let style = element.style();
                assert_eq!(
                    style.background,
                    Some(gpui::Fill::from(super::super::registry::hsla(expected.0)))
                );
                assert_eq!(
                    style.text.color,
                    Some(super::super::registry::hsla(expected.1))
                );
            }
            let cases = [
                (
                    "sidebar",
                    ThemeToken::SidebarBackground,
                    ThemeToken::SidebarForeground,
                ),
                (
                    "toolbar",
                    ThemeToken::ToolbarBackground,
                    ThemeToken::ToolbarForeground,
                ),
                (
                    "tab_bar",
                    ThemeToken::TabBarBackground,
                    ThemeToken::TabBarForeground,
                ),
            ];
            for (name, bg, fg) in cases {
                let expected = theme.surface_colors(name, &selector, bg, fg).unwrap();
                let resolved = theme.component_style(name, "body", &selector).unwrap();
                assert_eq!(expected.0, resolved.colors["background"]);
                assert_eq!(expected.1, resolved.colors["foreground"]);
                match name {
                    "sidebar" => check(
                        apply_surface(
                            Sidebar::<SidebarMenuItem>::new("sidebar-test"),
                            theme,
                            name,
                            &selector,
                            bg,
                            fg,
                        ),
                        expected,
                    ),
                    "toolbar" => check(
                        apply_surface(Toolbar::new("toolbar-test"), theme, name, &selector, bg, fg),
                        expected,
                    ),
                    _ => check(
                        apply_surface(TabBar::new("tabs-test"), theme, name, &selector, bg, fg),
                        expected,
                    ),
                }
            }
        });
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    use crate::theme::{
        activate_theme, parse_theme,
        registry::{active_theme, hsla},
    };
    use gpui::Styled as _;
    use gpui_component::{
        sidebar::{Sidebar, SidebarMenuItem},
        tab::TabBar,
        toolbar::Toolbar,
    };

    #[gpui::test]
    fn missing_surface_rules_apply_existing_role_tokens(cx: &mut gpui::TestAppContext) {
        let document = parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
                activate_theme(&document, Some(mode), app).unwrap();
                let theme = active_theme(app).unwrap();
                let selector = crate::components::theme::selector_for_mode(mode);
                fn check<T: gpui::Styled>(
                    mut widget: T,
                    theme: &ActiveTheme,
                    background: ThemeToken,
                    foreground: ThemeToken,
                ) {
                    assert_eq!(
                        widget.style().background,
                        Some(gpui::Fill::from(hsla(theme.token(background).unwrap())))
                    );
                    assert_eq!(
                        widget.style().text.color,
                        Some(hsla(theme.token(foreground).unwrap()))
                    );
                }
                let sidebar = apply_surface(
                    Sidebar::<SidebarMenuItem>::new("fallback-sidebar"),
                    theme,
                    "sidebar",
                    &selector,
                    ThemeToken::SidebarBackground,
                    ThemeToken::SidebarForeground,
                );
                check(
                    sidebar,
                    theme,
                    ThemeToken::SidebarBackground,
                    ThemeToken::SidebarForeground,
                );
                let toolbar = apply_surface(
                    Toolbar::new("fallback-toolbar"),
                    theme,
                    "toolbar",
                    &selector,
                    ThemeToken::ToolbarBackground,
                    ThemeToken::ToolbarForeground,
                );
                check(
                    toolbar,
                    theme,
                    ThemeToken::ToolbarBackground,
                    ThemeToken::ToolbarForeground,
                );
                for background in [
                    ThemeToken::TabBarBackground,
                    ThemeToken::TabBarSegmentedBackground,
                    ThemeToken::TabBarTransparentBackground,
                ] {
                    let tabs = apply_surface(
                        TabBar::new("fallback-tabs"),
                        theme,
                        "tab_bar",
                        &selector,
                        background,
                        ThemeToken::TabBarForeground,
                    );
                    check(tabs, theme, background, ThemeToken::TabBarForeground);
                }
            }
        });
    }
}
