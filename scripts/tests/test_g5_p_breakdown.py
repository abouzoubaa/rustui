import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from g5_p_breakdown import classify, exclusive


class PhaseBreakdownTests(unittest.TestCase):
    def test_nested_layout_is_counted_once_and_removed_from_prepaint(self):
        totals = exclusive([
            (0., .006, "prepaint"),
            (.001, .003, "layout"),
            (.002, .004, "layout"),
        ], ("layout", "prepaint"))
        self.assertAlmostEqual(totals["layout"], 3.)
        self.assertAlmostEqual(totals["prepaint"], 3.)

    def test_one_refresh_threshold_and_unobservable_keys(self):
        self.assertEqual(classify(.015625, .01953125, .0078125), "hit")
        self.assertEqual(classify(.015625, .0234375, .0078125), "miss")
        self.assertEqual(classify(.015625, .01171875, .0078125), "negative_clock_margin")
        self.assertEqual(classify(.015625, None, .0078125), "unobserved")
