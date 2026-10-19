"""P3D fixture tests. Native launch and process sampling are always mocked."""

import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import compare_cache_workload as comparison
import runtime_metrics


def workload(run, pid, protocol=None):
    protocol = protocol or comparison.WORKLOADS["8-leaf"]
    leaves = protocol["leaves"]
    leaf = 0 if run["mode"] == "cached" else leaves * protocol["measured_updates"]
    return {"schema_version": 1, "kind": "rustui_cache_comparison_workload", "run_id": run["run_id"],
            "pid": pid, "mode": run["mode"], "status": "complete", "ax_active_seen": False,
            "native_drag_seen": False, "environment_scope": "empty", "correctness_passed": True,
            "protocol": protocol, "values": [1] * leaves, "mounts": [1] * leaves,
            "counts": {"parent_root": protocol["measured_updates"], "parent_leaf": leaf,
                       "final_leaf": leaf + leaves, "mutation_leaf_delta": leaves}}


def raw_capture(pid=100, run_id="test", provenance=None, cpu=20):
    return {"pid": pid, "run_id": run_id, "identity": {"pid": pid, "lstart": "fixture start"},
            "provenance": provenance, "completion": {"status": "complete", "error": None},
            "samples": [{"elapsed_ns": 0, "ps_duration_ns": 1, "cpu_time_ns": 10, "rss_bytes": 100},
                        {"elapsed_ns": 100, "ps_duration_ns": 1, "cpu_time_ns": 10 + cpu, "rss_bytes": 200}]}


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "target/headstart/performance"
        self.output.mkdir(parents=True)
        for name in comparison.INPUTS:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(name)
        self.binary = self.root / "cache_comparison"
        self.binary.write_bytes(b"same native binary")
        for target, field, value in [(comparison, "ROOT", self.root), (comparison, "OUTPUT_DIR", self.output),
                                     (runtime_metrics, "OUTPUT_DIR", self.output)]:
            scope = patch.object(target, field, value)
            scope.start()
            self.addCleanup(scope.stop)
        compiler = patch.object(comparison, "compiler_identity", return_value={"sha256": "fixture compiler"})
        compiler.start()
        self.addCleanup(compiler.stop)

    def plan(self, workload="8-leaf", session="fixture"):
        path = comparison.make_plan(session, self.binary, workload)
        return path, comparison.load_plan(path)

    def fixture_captures(self, control_cpu=20):
        path, plan = self.plan()
        for run in plan["runs"]:
            pid = 1000 + run["index"]
            result_path = comparison.path_for(run["run_id"], "result")
            result = workload(run, pid, plan["protocol"])
            runtime_metrics.write_json_exclusive(result_path, result)
            final = {"schema_version": 1, "kind": "rustui_cache_comparison_final",
                     "run_id": run["run_id"], "mode": run["mode"], "pid": pid,
                     "status": "complete", "ax_active_seen": False,
                     "native_drag_seen": False, "workload": result}
            final_path = comparison.path_for(run["run_id"], "final")
            runtime_metrics.write_json_exclusive(final_path, final)
            cpu = 10000 if run["phase"] == "warmup" else (10 if run["mode"] == "cached" else control_cpu)
            raw = raw_capture(pid, run["run_id"], {"plan": comparison.identity(path)}, cpu)
            record = {"status": "complete", "run_id": run["run_id"], "pid": pid,
                      "mode": run["mode"], "phase": run["phase"], "pair": run["pair"],
                      "plan": comparison.identity(path), "executable": comparison.identity(self.binary),
                      "workload": result, "workload_artifact": comparison.identity(result_path),
                      "final_workload": final,
                      "final_workload_artifact": comparison.identity(final_path),
                      "raw_process_metrics": raw, "metrics": comparison.metric_values(raw)}
            runtime_metrics.write_json_exclusive(comparison.path_for(run["run_id"], "metrics"), record)
        return path, plan

    def test_fixed_schedule_alternates_pair_order_and_excludes_warmups(self):
        runs = comparison.schedule("session")
        self.assertEqual(len(runs), 12)
        self.assertEqual([r["mode"] for r in runs[:6]], ["cached", "uncached", "uncached", "cached", "cached", "uncached"])
        self.assertEqual(sum(r["phase"] == "warmup" for r in runs), 2)
        self.assertEqual(len({r["run_id"] for r in runs}), 12)

    def test_unsafe_session_ids_are_rejected(self):
        for value in ("", "../bad", "a/b", " space", "a" * 41):
            with self.subTest(value=value), self.assertRaises(ValueError):
                comparison.schedule(value)

    def test_plan_and_configuration_are_exclusive_and_record_hashes(self):
        path, plan = self.plan()
        self.assertEqual(plan["binary"]["sha256"], comparison.identity(self.binary)["sha256"])
        self.assertEqual(set(plan["inputs"]), set(comparison.INPUTS))
        for run in plan["runs"]:
            self.assertEqual(comparison.read_json(run["configuration_path"]), run["bundle_configuration"])
        with self.assertRaises(ValueError):
            comparison.make_plan("fixture", self.binary)
        self.assertTrue(path.exists())

    def test_workload_selection_preserves_shape_and_records_leaf_count(self):
        for name, leaves in (("8-leaf", 8), ("32-leaf", 32)):
            with self.subTest(workload=name):
                path, plan = self.plan(name, f"fixture-{name}")
                self.assertEqual(plan["protocol"], {"leaves": leaves, "lines_per_leaf": 12,
                                 "width": 210.0, "height": 220.0, "warmup_updates": 10,
                                 "measured_updates": 100})
                self.assertTrue(all(run["bundle_configuration"]["leaves"] == leaves for run in plan["runs"]))
                self.assertEqual(comparison.load_plan(path)["workload"], name)

    def test_unknown_workload_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown workload"):
            comparison.make_plan("fixture", self.binary, "64-leaf")

    def test_plan_detects_input_or_binary_drift(self):
        path, _ = self.plan()
        self.binary.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "binary changed"):
            comparison.load_plan(path)
        self.binary.write_bytes(b"same native binary")
        (self.root / comparison.INPUTS[0]).write_text("changed")
        with self.assertRaisesRegex(ValueError, "input changed"):
            comparison.load_plan(path)

    def test_plan_rejects_schedule_and_configuration_tampering(self):
        path, plan = self.plan()
        for field, value in (("mode", "uncached"), ("configuration_path", "/outside.json")):
            changed = copy.deepcopy(plan)
            changed["runs"][0][field] = value
            path.write_text(json.dumps(changed))
            with self.assertRaises(ValueError):
                comparison.load_plan(path)

    def test_correctness_rejects_exclusion_wrong_counts_values_and_mounts(self):
        for mode in ("cached", "uncached"):
            run = {"run_id": "fixture", "mode": mode}
            protocol = comparison.WORKLOADS["8-leaf"]
            good = workload(run, 7, protocol)
            comparison.check_workload(good, run, 7, protocol)
            for field, value in (("ax_active_seen", True), ("native_drag_seen", True), ("values", [0] * 8),
                                 ("mounts", [2] * 8), ("counts", {}), ("pid", 8), ("status", "timeout")):
                changed = {**good, field: value}
                with self.subTest(mode=mode, field=field), self.assertRaises(ValueError):
                    comparison.check_workload(changed, run, 7, protocol)

    def test_workload_invariants_scale_to_32_leaves(self):
        protocol = comparison.WORKLOADS["32-leaf"]
        for mode in ("cached", "uncached"):
            run = {"run_id": "fixture", "mode": mode}
            result = workload(run, 7, protocol)
            comparison.check_workload(result, run, 7, protocol)
            self.assertEqual(len(result["values"]), 32)
            self.assertEqual(len(result["mounts"]), 32)
            self.assertEqual(result["counts"]["mutation_leaf_delta"], 32)
        run = {"run_id": "fixture", "mode": "uncached"}
        bad = workload(run, 7, protocol)
        bad["counts"]["parent_leaf"] += 1
        with self.assertRaisesRegex(ValueError, "wrong raw lowering counts"):
            comparison.check_workload(bad, run, 7, protocol)

    def test_metric_values_use_attached_delta_and_sampled_rss(self):
        self.assertEqual(comparison.metric_values(raw_capture()), {"attached_cpu_ns": 20,
                         "attached_elapsed_ns": 100, "median_sampled_rss_bytes": 150, "max_sampled_rss_bytes": 200})

    def test_metrics_reject_partial_backwards_nonfinite_and_boolean_samples(self):
        for field, value in (("cpu_time_ns", 0), ("rss_bytes", float("nan")),
                             ("elapsed_ns", 0), ("ps_duration_ns", True)):
            raw = raw_capture()
            raw["samples"][-1][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                comparison.metric_values(raw)
        raw = raw_capture()
        raw["completion"]["status"] = "incomplete"
        with self.assertRaises(ValueError):
            comparison.metric_values(raw)

    def capture_fixture(self, status="complete", partial=False, final_guard=None, missing_final=False):
        path, plan = self.plan()
        run, pid = plan["runs"][0], 123
        ready = {"schema_version": 1, "kind": "rustui_cache_comparison_ready", "run_id": run["run_id"],
                 "mode": run["mode"], "pid": pid, "executable": str(self.binary)}
        runtime_metrics.write_json_exclusive(comparison.path_for(run["run_id"], "ready"), ready)

        def fake_collect(*args, provenance, stop_requested):
            self.assertFalse(stop_requested())
            self.assertTrue(comparison.path_for(run["run_id"], "start").exists())
            result = workload(run, pid, plan["protocol"])
            result["status"] = status
            runtime_metrics.write_json_exclusive(comparison.path_for(run["run_id"], "result"), result)
            self.assertFalse(stop_requested())
            self.assertTrue(stop_requested())
            raw = raw_capture(pid, run["run_id"], provenance)
            if partial:
                raw["completion"]["status"] = "incomplete"
            return raw

        def fake_sleep(_):
            if missing_final:
                return
            ax, drag = final_guard or (False, False)
            final = {"schema_version": 1, "kind": "rustui_cache_comparison_final",
                     "run_id": run["run_id"], "mode": run["mode"], "pid": pid,
                     "status": "excluded_interaction" if ax or drag else status,
                     "ax_active_seen": ax, "native_drag_seen": drag,
                     "workload": comparison.read_json(comparison.path_for(run["run_id"], "result"))}
            runtime_metrics.write_json_exclusive(comparison.path_for(run["run_id"], "final"), final)

        with patch.object(comparison.platform, "system", return_value="Darwin"), \
             patch.object(comparison, "inspect_process") as inspect, \
             patch.object(comparison.collector, "collect", side_effect=fake_collect), \
             patch.object(comparison.time, "sleep", side_effect=fake_sleep), \
             patch.object(comparison, "FINAL_SNAPSHOT_TIMEOUT", 0 if missing_final else comparison.FINAL_SNAPSHOT_TIMEOUT):
            destination, outcome = comparison.collect_run(path, 0, pid)
        inspect.assert_called_once_with(pid, self.binary.resolve())
        self.assertTrue(comparison.path_for(run["run_id"], "ack").exists())
        return comparison.read_json(destination), outcome

    def test_collector_gates_native_process_and_keeps_raw_samples(self):
        record, outcome = self.capture_fixture()
        self.assertEqual(outcome, "complete")
        self.assertEqual(len(record["raw_process_metrics"]["samples"]), 2)

    def test_collector_preserves_excluded_and_partial_results(self):
        record, outcome = self.capture_fixture("excluded_interaction")
        self.assertEqual(outcome, "excluded")
        self.assertIn("error", record)

    def test_collector_preserves_partial_sampling_failure(self):
        record, outcome = self.capture_fixture(partial=True)
        self.assertEqual(outcome, "failed")
        self.assertEqual(record["raw_process_metrics"]["completion"]["status"], "incomplete")

    def test_late_ax_guard_excludes_run(self):
        record, outcome = self.capture_fixture(final_guard=(True, False))
        self.assertEqual(outcome, "excluded")
        self.assertEqual(len(record["raw_process_metrics"]["samples"]), 2)

    def test_late_native_drag_guard_excludes_run(self):
        record, outcome = self.capture_fixture(final_guard=(False, True))
        self.assertEqual(outcome, "excluded")
        self.assertEqual(len(record["raw_process_metrics"]["samples"]), 2)

    def test_missing_final_snapshot_fails_after_bounded_wait_and_keeps_samples(self):
        record, outcome = self.capture_fixture(missing_final=True)
        self.assertEqual(outcome, "failed")
        self.assertEqual(len(record["raw_process_metrics"]["samples"]), 2)
        self.assertIn("FileNotFoundError", record["error"])

    def test_summary_is_paired_and_warmup_metrics_do_not_affect_measured_median(self):
        path, _ = self.fixture_captures()
        output = comparison.summarize(path)
        report = comparison.read_json(output)
        self.assertEqual(report["workload"], "8-leaf")
        self.assertEqual(report["protocol"]["leaves"], 8)
        self.assertEqual(report["measured_pairs"], 5)
        self.assertEqual(report["modes"]["cached"]["attached_cpu_ns"]["median"], 10)
        self.assertEqual(report["ratio_spread"]["attached_cpu_ns"]["median"], .5)
        with self.assertRaises(FileExistsError):
            comparison.summarize(path)

    def test_summary_rejects_raw_counter_tampering(self):
        path, plan = self.fixture_captures()
        target = comparison.path_for(plan["runs"][2]["run_id"], "metrics")
        record = comparison.read_json(target)
        record["metrics"]["attached_cpu_ns"] = 999
        target.write_text(json.dumps(record))
        with self.assertRaisesRegex(ValueError, "raw samples"):
            comparison.summarize(path)

    def test_summary_rejects_changed_result_artifact(self):
        path, plan = self.fixture_captures()
        target = comparison.path_for(plan["runs"][2]["run_id"], "result")
        target.write_text("{}")
        with self.assertRaisesRegex(ValueError, "artifact changed"):
            comparison.summarize(path)

    def test_summary_rejects_changed_final_artifact(self):
        path, plan = self.fixture_captures()
        record = comparison.read_json(comparison.path_for(plan["runs"][2]["run_id"], "metrics"))
        Path(record["final_workload_artifact"]["path"]).write_text("{}")
        with self.assertRaisesRegex(ValueError, "final artifact changed"):
            comparison.summarize(path)

    def test_summary_rejects_reused_pid_or_failed_capture(self):
        path, plan = self.fixture_captures()
        target = comparison.path_for(plan["runs"][2]["run_id"], "metrics")
        record = comparison.read_json(target)
        record["pid"] = 1000
        target.write_text(json.dumps(record))
        with self.assertRaisesRegex(ValueError, "process identities"):
            comparison.summarize(path)

    def test_zero_control_is_unavailable_not_an_infinite_speedup(self):
        self.assertIsNone(comparison.spread([0, 0])["range_percent_of_median"])
        for values in ([], [float("inf")], [-1]):
            with self.assertRaises(ValueError):
                comparison.spread(values)

    def test_summary_marks_zero_cpu_denominator_unavailable(self):
        path, _ = self.fixture_captures(control_cpu=0)
        summary = comparison.read_json(comparison.summarize(path))
        self.assertEqual(summary["ratio_spread"]["attached_cpu_ns"], {"status": "unavailable_zero_control"})
        self.assertTrue(all(pair["cached_over_uncached"]["attached_cpu_ns"] is None for pair in summary["paired_ratios"]))


if __name__ == "__main__":
    unittest.main()
