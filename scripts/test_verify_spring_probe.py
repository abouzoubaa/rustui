"""Focused validation cases for the spring probe result contract."""

from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from verify_spring_probe import validate


def valid_result(reduced: bool) -> dict:
    counts = (
        [1, 1, 2, 2, 2, 2]
        if reduced else [3, 13, 15, 26, 42, 42]
    )
    return {
        "kind": "rustui_spring_opacity_scheduling_probe",
        "status": "complete",
        "mode": "reduce_motion" if reduced else "normal",
        "error": None,
        "requested_reduce_motion": reduced,
        "effective_reduce_motion": reduced,
        "target_sequence": [0.15000000596046448, 1.0, 0.15000000596046448],
        "startup_stability_ms": 750,
        "startup_timeout_ms": 3000,
        "startup_wait_ms": 795,
        "startup_stable_lowerings": 2,
        "poll_interval_ms": 25,
        "retarget_after_first_render_ms": 250,
        "passive_after_second_render_ms": 250,
        "settle_after_retarget_ms": 2000,
        "quiescence_window_ms": 750,
        "process_timeout_ms": 10000,
        "timings_monotonic": True,
        "quiescent": True,
        "measurement_scope": "RustUI root host lowerings only; not exact GPUI frames, presented alpha, or spring velocity continuity",
        "event_elapsed_ms": {"first_update": 1331, "retarget": 1641},
        "sample_elapsed_ms": {
            "first_render": 1357,
            "before_retarget": 1641,
            "second_render": 1693,
            "after_retarget": 1966,
            "settled": 3723,
            "quiescence": 4511,
        },
        "root_host_lowerings": dict(zip((
            "first_render", "before_retarget", "second_render",
            "after_retarget", "settled", "quiescence",
        ), counts)),
        "first_motion_growth": not reduced,
        "retarget_motion_growth": not reduced,
        "first_passive_flat": reduced,
        "second_passive_flat": reduced,
    }


class SpringVerifierTests(unittest.TestCase):
    def test_accepts_valid_normal_and_reduced_records(self) -> None:
        for reduced in (False, True):
            result = valid_result(reduced)
            self.assertEqual(validate(result, result["mode"]), [])

    def test_rejects_wrong_or_nonfinite_targets(self) -> None:
        for invalid in (0.151, True, "0.15", float("nan"), float("inf")):
            result = valid_result(False)
            result["target_sequence"][0] = invalid
            with self.subTest(invalid=invalid):
                self.assertTrue(validate(result, "normal"))

    def test_rejects_short_or_reversed_timing(self) -> None:
        for key, value in (("before_retarget", 1600), ("settled", 3600), ("quiescence", 4300)):
            result = valid_result(False)
            result["sample_elapsed_ms"][key] = value
            with self.subTest(key=key):
                self.assertTrue(validate(result, "normal"))

    def test_rejects_flatness_growth_and_count_mutations_despite_complete_status(self) -> None:
        mutations = (
            (False, "before_retarget", 3),
            (False, "after_retarget", 15),
            (False, "quiescence", 43),
            (True, "before_retarget", 2),
            (True, "after_retarget", 3),
            (True, "second_render", 1),
        )
        for reduced, key, value in mutations:
            result = deepcopy(valid_result(reduced))
            result["root_host_lowerings"][key] = value
            with self.subTest(reduced=reduced, key=key):
                self.assertEqual(result["status"], "complete")
                self.assertTrue(validate(result, result["mode"]))


if __name__ == "__main__":
    unittest.main()
