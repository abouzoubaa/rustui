import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "verify_theme_adapter_coverage.py"
SPEC = importlib.util.spec_from_file_location("verify_theme_adapter_coverage", SCRIPT)
coverage = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(coverage)


class RendererCoverageTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
