"""G0 frame summaries must not hide verification failures outside timed samples."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
SCENES = ("idle", "typing", "hover", "log", "drag")


class G0BenchTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="rustui-g0-summary-")
        self.addCleanup(self.temporary.cleanup)
        self.output = Path(self.temporary.name)
        for scene in SCENES:
            start = {"event": "start", "scene": scene, "epoch_ms": 1000}
            end = {"event": "end", "scene": scene, "epoch_ms": 61000, "valid": True}
            lines = ["RUSTUI_G0_LOG " + json.dumps(start)]
            for index in range(100):
                mode = "copy" if index == 0 else "partial" if index == 1 else "direct"
                records = [
                    {"event": "draw", "cpu_ms": 0.5, "epoch_ms": 2000 + index},
                    {"event": "present", "cpu_ms": 0.2, "epoch_ms": 2000 + index},
                    {"event": "gpu", "mode": mode, "gpu_ms": 0.25,
                     "cpu_ms": 0.0, "epoch_ms": 2000 + index},
                ]
                lines.extend("RUSTUI_FRAME_LOG " + json.dumps(record) for record in records)
            lines.append("RUSTUI_FRAME_LOG " + json.dumps(
                {"event": "gpu", "mode": "skip", "gpu_ms": None,
                 "cpu_ms": 0.0, "epoch_ms": 3000}))
            lines.append("RUSTUI_G0_LOG " + json.dumps(end))
            (self.output / f"{scene}-frames.log").write_text("\n".join(lines) + "\n")

    def summarize(self, damage="verify"):
        environment = dict(os.environ, RUSTUI_DAMAGE=damage)
        return subprocess.run(
            ["bash", "-c", 'source "$1"; g0_summarize "$2"', "g0-summary-test",
             str(REPO / "scripts/run_g0_bench.sh"), str(self.output)],
            cwd=REPO, env=environment, text=True, capture_output=True, check=False,
        )

    def append_mismatch(self, epoch_ms):
        path = self.output / "idle-frames.log"
        with path.open("a") as stream:
            stream.write("RUSTUI_FRAME_LOG " + json.dumps(
                {"event": "damage_mismatch", "mode": "partial", "x": 3, "y": 7,
                 "cpu_ms": 0.0, "epoch_ms": epoch_ms}) + "\n")

    def test_warmup_mismatch_rejects_complete_capture(self):
        self.append_mismatch(500)
        result = self.summarize()
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("damage mismatch anywhere in scene log", result.stderr)
        self.assertFalse((self.output / "summary.json").exists())

    def test_mismatch_after_end_marker_rejects_complete_capture(self):
        self.append_mismatch(62000)
        result = self.summarize()
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("damage mismatch anywhere in scene log", result.stderr)
        self.assertFalse((self.output / "summary.json").exists())

    def test_valid_capture_counts_skip_without_gpu_time(self):
        result = self.summarize()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        rows = json.loads((self.output / "summary.json").read_text())
        self.assertEqual(len(rows), 5)
        for row in rows:
            self.assertEqual(row["frame_mode_counts"],
                             {"skip": 1, "copy": 1, "partial": 1, "retained": 0, "direct": 98})
            self.assertEqual(row["frame_mode_samples"], 101)
            self.assertEqual(row["gpu_samples"], 100)
            self.assertEqual(row["gpu_mean_ms"], 0.25)

    def test_damage_cpu_is_separate_from_cpu_phase_totals(self):
        for scene in SCENES:
            with (self.output / f"{scene}-frames.log").open("a") as stream:
                stream.write("RUSTUI_FRAME_LOG " + json.dumps(
                    {"event": "damage_cpu", "cpu_ms": 0.0, "epoch_ms": 3000,
                     "signatures_ms": 0.1, "plan_ms": 0.2, "total_ms": 0.35,
                     "hashed": 2, "reused": 10, "reason": "partial"}) + "\n")
        result = self.summarize()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for row in json.loads((self.output / "summary.json").read_text()):
            self.assertEqual(row["damage_cpu"],
                             {"samples": 1, "signatures_mean_ms": 0.1,
                              "plan_mean_ms": 0.2, "total_mean_ms": 0.35,
                              "hashed": 2, "reused": 10, "reasons": {"partial": 1}})
            self.assertNotIn("damage_cpu", row["cpu_phases_mean_ms"])
            self.assertAlmostEqual(row["cpu_mean_ms"], 0.7)

    def test_matched_pair_verdicts_use_every_pair_without_spread(self):
        off = [1.0, 1.01, 1.02, 1.03, 1.04]
        ratios = [(1.05,) * 5, (1.02, 1.04, 1.01, 1.05, 1.02),
                  (1.03,) * 5, (0.98,) * 5, (1.001,) * 5]
        for scene, multipliers in zip(SCENES, ratios):
            for mode, values in (("0", off), ("1", [a*b for a,b in zip(off, multipliers)])):
                for run, value in enumerate(values, 1):
                    folder = self.output / scene / f"{run}-{mode}"
                    folder.mkdir(parents=True)
                    (folder / "summary.json").write_text(json.dumps([
                        {"scene": scene, "damage": mode, "cpu_mean_ms": value}]))
        result = subprocess.run(
            ["bash", "-c", 'source "$1"; g0_interleaved_summary "$2"', "g0-matched-test",
             str(REPO / "scripts/run_g0_bench.sh"), str(self.output)],
            cwd=REPO, text=True, capture_output=True, check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        rows = json.loads((self.output / "interleaved-summary.json").read_text())
        self.assertEqual([r["verdict"] for r in rows],
                         ["FAIL", "INCONCLUSIVE", "PASS", "PASS", "PASS"])
        self.assertAlmostEqual(rows[0]["off"]["median_ms"], 1.02)
        for actual, expected in zip(rows[1]["paired_changes_pct"], (2,4,1,5,2)):
            self.assertAlmostEqual(actual, expected)
        self.assertAlmostEqual(rows[1]["median_change_pct"], 2)
        self.assertAlmostEqual(rows[1]["min_change_pct"], 1)
        self.assertAlmostEqual(rows[1]["max_change_pct"], 5)

    def test_damage_off_rejects_non_direct_modes(self):
        result = self.summarize("0")
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("damage off must draw Direct", result.stderr)


if __name__ == "__main__":
    unittest.main()
