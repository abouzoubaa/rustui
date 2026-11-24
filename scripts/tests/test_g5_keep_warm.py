import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from g5_keep_warm import active_link_interval, overlapping_late_ticks, refresh_group, update_intervals


class KeepWarmAnalysisTests(unittest.TestCase):
    def test_uses_display_bracket_and_last_completed_interval_separately(self):
        previous, age, around = update_intervals([0.0, .01, .025, .04], .02)
        self.assertAlmostEqual(previous, .01)
        self.assertAlmostEqual(age, .01)
        self.assertAlmostEqual(around, .015)
        self.assertEqual(refresh_group(previous, age), "slower_update")
        self.assertEqual(refresh_group(previous, .080), "unobserved")
        self.assertEqual(refresh_group(1 / 120, .003), "near_120_update")

    def test_inactive_link_and_nonoverlapping_stall_have_no_observations(self):
        self.assertIsNone(active_link_interval([.01, .02], [.02, .03], .09))
        self.assertAlmostEqual(active_link_interval([.01, .02], [.02, .03], .021), .01)
        self.assertEqual(overlapping_late_ticks([(.003, .009), (.011, .016), (.021, .025)],
                                               .010, .020), [(.011, .016)])
