import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

SCRIPT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPT_DIR))
import summarize_runtime_metrics as summary


def record():
    return {
        "schema_version": 1,
        "kind": "rustui_process_metrics",
        "run_id": "known-run",
        "label": "idle baseline",
        "workload": "idle-v1",
        "pid": 123,
        "identity": {"pid": 123, "lstart": "Mon Oct 18 12:00:00 2026"},
        "completion": {"status": "complete", "error": None},
        "metrics_availability": {"cycle_time": "unavailable", "frame_time": "unavailable", "main_thread_time": "unavailable"},
        "samples": [
            {"elapsed_ns": 0, "ps_duration_ns": 2_000_000, "cpu_time_ns": 10_000_000, "rss_bytes": 100},
            {"elapsed_ns": 1_000_000_000, "ps_duration_ns": 3_000_000, "cpu_time_ns": 1_210_000_000, "rss_bytes": 400},
            {"elapsed_ns": 2_000_000_000, "ps_duration_ns": 1_000_000, "cpu_time_ns": 2_410_000_000, "rss_bytes": 200},
        ],
    }


class SummaryTests(unittest.TestCase):
    def test_known_metrics_allow_multi_cpu_percent(self):
        result = summary.summarize_record(record())
        self.assertEqual(result["mean_process_cpu_percent"], 120.0)
        self.assertEqual(result["cpu_total_delta_ns"], 2_400_000_000)
        self.assertEqual(result["observed_sample_elapsed_ns"], 2_000_000_000)
        self.assertEqual(result["rss_bytes"], {"min": 100, "max": 400, "median": 200})
        self.assertEqual(result["sampling_overhead_ms"], {"median": 2.0, "p95_nearest_rank": 3.0})
        self.assertEqual(result["frame_time"], "unavailable")

    def test_incomplete_record_is_reported_incomplete(self):
        source = record()
        source["completion"] = {"status": "incomplete", "error": "process exited"}
        self.assertEqual(summary.build_report([source])["runs"][0]["completion_status"], "incomplete")

    def test_rejects_bad_sample_values(self):
        cases = [
            ("elapsed_ns", [0, 0, 2]),
            ("cpu_time_ns", [10, 9, 12]),
            ("rss_bytes", [100, -1, 200]),
            ("ps_duration_ns", [2, float("nan"), 1]),
            ("elapsed_ns", [0, True, 2]),
        ]
        for field, values in cases:
            with self.subTest(field=field, values=values):
                source = record()
                for sample, value in zip(source["samples"], values):
                    sample[field] = value
                with self.assertRaises(summary.RecordError):
                    summary.summarize_record(source)

    def test_rejects_bool_pid(self):
        source = record()
        source["pid"] = True
        with self.assertRaises(summary.RecordError):
            summary.summarize_record(source)

    def test_rejects_bool_schema_version_and_invalid_json_constants(self):
        source = record()
        source["schema_version"] = True
        with self.assertRaises(summary.RecordError):
            summary.summarize_record(source)
        with tempfile.TemporaryDirectory() as directory:
            raw = Path(directory) / "raw.json"
            raw.write_text('{"sample": NaN}', encoding="utf-8")
            with self.assertRaisesRegex(summary.RecordError, "non-finite JSON number"):
                summary.read_record(raw)

    def test_rejects_negative_first_timestamp_or_cpu_and_noninteger_units(self):
        for field, value in (("elapsed_ns", -1), ("cpu_time_ns", -1), ("rss_bytes", 1.5), ("ps_duration_ns", True)):
            with self.subTest(field=field):
                source = record()
                source["samples"][0][field] = value
                with self.assertRaises(summary.RecordError):
                    summary.summarize_record(source)

    def test_incomplete_record_without_samples_is_rejected_explicitly(self):
        source = record()
        source["completion"] = {"status": "incomplete", "error": "first sample failed"}
        source["samples"] = []
        with self.assertRaisesRegex(summary.RecordError, "non-empty list"):
            summary.summarize_record(source)

    def test_rejects_missing_identity_and_contradictory_completion(self):
        source = record()
        source["identity"] = None
        with self.assertRaisesRegex(summary.RecordError, "identity"):
            summary.summarize_record(source)
        source = record()
        source["completion"]["error"] = "unexpected error"
        with self.assertRaisesRegex(summary.RecordError, "complete records require"):
            summary.summarize_record(source)

    def test_output_uses_exclusive_writer_and_refuses_existing_path(self):
        source = record()
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "summary.json"
            raw = Path(directory) / "raw.json"
            raw.write_text(json.dumps(source), encoding="utf-8")
            output.write_text("keep", encoding="utf-8")
            with mock.patch.object(summary, "validate_output_path", return_value=output):
                with mock.patch.object(summary, "write_json_exclusive", side_effect=FileExistsError("exists")) as writer:
                    with self.assertRaises(SystemExit) as error:
                        summary.main([str(raw), "--output", str(output)])
                    self.assertEqual(error.exception.code, 2)
                    writer.assert_called_once()
            self.assertEqual(output.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
