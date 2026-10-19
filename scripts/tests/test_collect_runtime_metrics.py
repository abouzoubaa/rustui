"""Focused checks for process counter parsing, partial runs, and exclusive output."""

import json
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import collect_runtime_metrics as collector
import runtime_metrics


class FakeClock:
    def __init__(self):
        self.now = 0

    def monotonic_ns(self):
        return self.now

    def sleep(self, seconds):
        self.now += round(seconds * 1_000_000_000)


class CollectorTests(unittest.TestCase):
    def test_cpu_clock_exact_and_large(self):
        self.assertEqual(collector.parse_cpu_time("0:00.01"), 10_000_000)
        self.assertEqual(collector.parse_cpu_time("12-23:59:59.123456789"),
                         ((12 * 24 + 23) * 3600 + 59 * 60 + 59) * 1_000_000_000 + 123_456_789)
        self.assertEqual(collector.parse_cpu_time("120:00"), 7_200_000_000_000)
        for bad in ("1-20:00", "0:60", "1:60:00", "1-24:00:00", "1:00.1234567891", "NaN"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                collector.parse_cpu_time(bad)

    def test_ps_row_identity_and_rss(self):
        row = "85682 Sun Oct 18 05:28:48 2026       0:00.01   2736\n"
        self.assertEqual(collector.parse_ps_row(row, 85682),
                         ("Sun Oct 18 05:28:48 2026", 10_000_000, 2_801_664))
        for bad in (row + row, row.replace("85682", "8"), row.replace("Sun", "Mon"),
                    row.replace("2736", "bad")):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                collector.parse_ps_row(bad, 85682)

    def test_positive_duration_rejects_nonfinite_and_subnanosecond(self):
        self.assertEqual(collector.positive_ns("0.000000001"), 1)
        for bad in ("0", "-1", "NaN", "sNaN", "Infinity", "0.0000000001"):
            with self.subTest(bad=bad), self.assertRaises(Exception):
                collector.positive_ns(bad)

    def test_pid_change_and_timeout_keep_partial_samples(self):
        for second in (("different start", {"elapsed_ns": 1}),
                       ("Sun Oct 18 05:28:48 2026", {"cpu_time_ns": 0}),
                       subprocess.TimeoutExpired("ps", 5)):
            clock = FakeClock()
            first = ("Sun Oct 18 05:28:48 2026",
                     {"elapsed_ns": 0, "ps_duration_ns": 1, "cpu_time_ns": 1, "rss_bytes": 1})
            with self.subTest(second=second), \
                 patch.object(collector.time, "monotonic_ns", clock.monotonic_ns), \
                 patch.object(collector.time, "sleep", clock.sleep), \
                 patch.object(collector, "sample", side_effect=[first, second]):
                result = collector.collect(1, 2_000_000_000, 1_000_000_000, "idle", "idle", "run")
            self.assertEqual(result["completion"]["status"], "incomplete")
            self.assertEqual(len(result["samples"]), 1)
            self.assertIsNotNone(result["completion"]["error"])

    def test_initial_ps_failure_keeps_zero_sample_report(self):
        with patch.object(collector, "sample", side_effect=subprocess.TimeoutExpired("ps", 5)):
            result = collector.collect(1, 1_000_000_000, 1_000_000_000, "idle", "idle", "run")
        self.assertEqual(result["completion"]["status"], "incomplete")
        self.assertEqual(result["samples"], [])
        self.assertIsNone(result["identity"])

    def test_interval_longer_than_duration_rejected(self):
        with redirect_stderr(StringIO()), self.assertRaises(SystemExit) as error:
            collector.main(["--pid", "1", "--duration", "1", "--interval", "2",
                            "--label", "idle", "--workload", "idle"])
        self.assertEqual(error.exception.code, 2)

    def test_overrun_skips_missed_ticks(self):
        clock = FakeClock()

        def slow_sample(pid, start_ns):
            clock.now += 2_500_000_000
            return "Sun Oct 18 05:28:48 2026", {
                "elapsed_ns": clock.now, "ps_duration_ns": 2_500_000_000,
                "cpu_time_ns": clock.now, "rss_bytes": 1024,
            }

        with patch.object(collector.time, "monotonic_ns", clock.monotonic_ns), \
             patch.object(collector.time, "sleep", clock.sleep), \
             patch.object(collector, "sample", side_effect=slow_sample):
            result = collector.collect(1, 5_000_000_000, 1_000_000_000, "idle", "idle", "run")
        self.assertEqual(result["completion"]["status"], "complete")
        self.assertEqual(len(result["samples"]), 2)

    def test_exclusive_writer_and_symlink_guard(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "repo"
            output_dir = root / "target/headstart/performance"
            output_dir.mkdir(parents=True)
            with patch.object(runtime_metrics, "OUTPUT_DIR", output_dir):
                path = runtime_metrics.default_output_path("safe", "process")
                runtime_metrics.write_json_exclusive(path, {"value": 1})
                self.assertEqual(json.loads(path.read_text()), {"value": 1})
                with self.assertRaises(FileExistsError):
                    runtime_metrics.write_json_exclusive(path, {"value": 2})
                with self.assertRaises(ValueError):
                    runtime_metrics.validate_output_path(root / "elsewhere.json")
                link = output_dir / "linked.json"
                link.symlink_to(path)
                with self.assertRaises(ValueError):
                    runtime_metrics.write_json_exclusive(link, {"value": 3})


class StopRequestedTests(unittest.TestCase):
    def test_stop_request_keeps_final_sample(self):
        clock = FakeClock()
        def point(pid, start_ns):
            return "Sun Oct 18 05:28:48 2026", {
                "elapsed_ns": clock.now, "ps_duration_ns": 1,
                "cpu_time_ns": clock.now, "rss_bytes": 1024,
            }
        with patch.object(collector.time, "monotonic_ns", clock.monotonic_ns), \
             patch.object(collector.time, "sleep", clock.sleep), \
             patch.object(collector, "sample", side_effect=point):
            result = collector.collect(1, 10_000_000_000, 1_000_000_000,
                                       "W1", "W1", "stop-test",
                                       stop_requested=lambda: clock.now >= 1_000_000_000)
        self.assertEqual(result["completion"]["status"], "complete")
        self.assertEqual(len(result["samples"]), 2)
        self.assertEqual(result["samples"][-1]["elapsed_ns"], 1_000_000_000)


if __name__ == "__main__":
    unittest.main()
