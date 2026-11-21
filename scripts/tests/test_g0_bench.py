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
            end = {"event": "end", "scene": scene, "epoch_ms": 61000, "valid": True,
                   "cpu_user_s": 1.0, "cpu_sys_s": 0.25}
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
            self.assertEqual(row["process_cpu_s"], 1.25)
            self.assertEqual(row["process_cpu_s_per_attempt"], 0.0125)
            self.assertEqual(row["frames_over_8_3_ms"], 0)

    def test_damage_off_rejects_non_direct_modes(self):
        result = self.summarize("0")
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("damage off must draw Direct", result.stderr)


if __name__ == "__main__":
    unittest.main()
