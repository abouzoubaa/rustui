import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("f0_scroll_capture", ROOT / "scripts/run_f0_scroll_capture.py")
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


def event(kind, cpu, gap=None):
    return "RUSTUI_FRAME_LOG " + json.dumps({"event": kind, "cpu_ms": cpu, "gap_ms": gap})


class ScrollCaptureTests(unittest.TestCase):
    def test_counts_draw_and_submit_together_and_excludes_idle_gap(self):
        log = "\n".join([event("present", 99, 9999), "RUSTUI_SCROLL_LOG start collection",
                          event("draw", 8), event("present", 1, 2000),
                          event("draw", 16), event("present", 1, 20),
                          "RUSTUI_SCROLL_LOG end collection events=2"])
        result = capture.summarize(log)
        self.assertEqual(result["cpu_over_8_3_ms"], 2)
        self.assertEqual(result["cpu_over_16_7_ms"], 1)
        self.assertEqual(result["worst_gap_ms"], 20)
        self.assertEqual(result["gaps_over_16_7_ms"], 1)
        self.assertIsNone(result["visible_rows_painted_empty"])

    def test_missing_or_incomplete_capture_is_rejected(self):
        for log in ["", "RUSTUI_SCROLL_LOG start collection", "RUSTUI_SCROLL_LOG end collection"]:
            with self.subTest(log=log), self.assertRaisesRegex(ValueError, "incomplete native capture"):
                capture.summarize(log)
