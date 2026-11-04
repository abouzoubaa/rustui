import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "verify_theme_adapter_coverage.py"
SPEC = importlib.util.spec_from_file_location("verify_theme_adapter_coverage", SCRIPT)
coverage = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(coverage)


class RendererCoverageTests(unittest.TestCase):
    def test_menu_theme_uses_activation_and_upstream_popover(self):
        registry = coverage.REGISTRY.read_text(encoding="utf-8")
        popup_path = "@gpui/menu/popup_menu.rs"
        styled_path = "@gpui/styled.rs"
        popup = (coverage.ROOT / "vendor/gpui-component-0.7.3/src/menu/popup_menu.rs").read_text()
        styled = (coverage.ROOT / "vendor/gpui-component-0.7.3/src/styled.rs").read_text()
        menu_path = "components/overlay/menu.rs"
        menu = (coverage.ROOT / "crates/rustui/src" / menu_path).read_text()
        diagnostic = "menu activation-to-popover theme path is incomplete"
        self.assertNotIn(diagnostic, coverage.check_renderer_calls())
        self.assertNotIn(
            "menu is not queried by components/overlay/menu.rs",
            coverage.check_renderer_calls(),
        )
        for path, original, old, new in [
            ("theme/registry.rs", registry, '"menu",\n        "body"', '"menu",\n        "other"'),
            ("theme/registry.rs", registry, "fn menu_background(", "fn removed_menu_background("),
            ("theme/registry.rs", registry, "colors.popover = opaque_popover_fallback", "colors.other = opaque_popover_fallback"),
            (menu_path, menu, ".dropdown_menu(", ".other_menu("),
            (popup_path, popup, ".popover_style(cx)", ".other_style(cx)"),
            (styled_path, styled, ".bg(theme.popover)", ".bg(theme.other)"),
        ]:
            with self.subTest(path=path, marker=old):
                self.assertIn(old, original)
                errors = coverage.check_renderer_calls({path: original.replace(old, new, 1)})
                self.assertIn(diagnostic, errors)

    def test_cfg_test_markers_cannot_stand_in_for_label_renderer(self):
        source = '''
#[cfg(test)]
mod tests {
    fn fixture() {
        let style = required_style(context, "label", "body", &query);
        styled_element();
    }
}
'''

        errors = coverage.check_renderer_calls(
            {"components/basic/label.rs": source}
        )

        self.assertIn("label is not queried by components/basic/label.rs", errors)
        self.assertIn(
            "label has no resolved-style consumer in components/basic/label.rs",
            errors,
        )

    def test_slider_track_requires_role_resolution_and_contrast_fallback(self):
        path = "components/input/slider.rs"
        source = (coverage.ROOT / "crates/rustui/src" / path).read_text(encoding="utf-8")
        diagnostic = "slider role `track` is not proven resolved and applied by source markers"
        self.assertNotIn(diagnostic, coverage.check_control_style_consumption())
        for original, replacement in [
            ("let track = slider_role_color(style, role, profile, mode);", "let track = other;"),
            ("if track == background {", "if false {"),
        ]:
            self.assertIn(original, source)
            mutated = source.replace(original, replacement, 1)
            errors = coverage.check_control_style_consumption(source_overrides={path: mutated})
            self.assertIn(diagnostic, errors)


if __name__ == "__main__":
    unittest.main()
