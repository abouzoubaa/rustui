import unittest

from scripts.calibrate_cache_workload import SCENARIOS, check_workload


class CalibrateCacheWorkloadTests(unittest.TestCase):
    def workload(self, scenario):
        count = SCENARIOS[scenario]
        return {
            "schema_version": 1,
            "kind": "rustui_cached_child_workload",
            "status": "complete",
            "ax_active_seen": False,
            "warmup_parent_updates": 10,
            "measured_parent_updates": 100,
            "scenario": scenario,
            "retained_children_per_side": count,
            "cache_hit_observed": True,
            "child_invalidation_observed": True,
            "counts": {
                "cached_after_parent_updates": 0,
                "uncached_after_parent_updates": count * 100,
            },
            "rendered_values": {"cached": 1, "uncached": 1},
        }

    def test_scenarios_have_distinct_retained_child_counts(self):
        self.assertEqual(SCENARIOS, {"small": 1, "medium": 8, "large": 32})

    def test_accepts_each_scenario_with_its_expected_work(self):
        for scenario in SCENARIOS:
            with self.subTest(scenario=scenario):
                check_workload(self.workload(scenario), scenario)

    def test_rejects_count_mismatch(self):
        result = self.workload("medium")
        result["counts"]["uncached_after_parent_updates"] = 100
        with self.assertRaisesRegex(ValueError, "uncached_after_parent_updates"):
            check_workload(result, "medium")

    def test_rejects_scenario_mismatch(self):
        result = self.workload("small")
        result["scenario"] = "large"
        with self.assertRaisesRegex(ValueError, "scenario"):
            check_workload(result, "small")


if __name__ == "__main__":
    unittest.main()
