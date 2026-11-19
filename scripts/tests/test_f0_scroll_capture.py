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

    def test_phase_profile_sums_layout_and_rows_without_calling_them_extra_draw_time(self):
        log = "\n".join(["RUSTUI_SCROLL_LOG start collection",
            event("build", 1), event("layout", .2), event("layout", .3),
            'RUSTUI_ROW_PROFILE {"total_ms":2,"lower_ms":1}',
            event("prepaint", 4), event("paint", 1), event("draw", 6), event("present", .1),
            "RUSTUI_SCROLL_LOG end collection"])
        result = capture.profile(log)
        self.assertEqual(result["drawn_frames"], 1)
        self.assertEqual(result["median_ms"]["layout"], .5)
        self.assertEqual(result["median_ms"]["row_total_ms"], 2)
        self.assertEqual(result["median_ms"]["draw"], 6)
        with self.assertRaisesRegex(ValueError, "missing native phase"):
            capture.profile("RUSTUI_SCROLL_LOG start collection\n" + event("draw", 6) + "\n" + event("present", .1))

    def test_visual_oracle_carries_idle_state_but_never_invents_a_new_scroll_position(self):
        painted = dict(epoch_ms=900, source=dict(offset_y=0,visual_offset_y=-12))
        logical = [dict(epoch_ms=1000,source=dict(offset_y=0)),dict(epoch_ms=1010,source=dict(offset_y=-100))]
        result = capture.visual_sources(logical,[painted])
        self.assertEqual(len(result),2)
        self.assertEqual(result[-1]["epoch_ms"],1000)
        self.assertEqual(result[-1]["source"]["visual_offset_y"],-12)

    def test_static_sidebar_cannot_validate_stale_workload_rows(self):
        entries = [dict(epoch_ms=t,entity_id=entity,source=dict(offset_y=offset))
                   for t,entity,offset in [(1,1,0),(1,2,0),(2,1,0),(2,2,-300)]]
        self.assertEqual([entry["entity_id"] for entry in capture.workload_sources(entries)],[2,2])
        with self.assertRaisesRegex(ValueError,"one moving workload"):
            capture.workload_sources(entries[:2])
