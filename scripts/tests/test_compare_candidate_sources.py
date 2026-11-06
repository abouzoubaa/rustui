"""Focused tests for read-only P3H source inventory comparison."""

from __future__ import annotations

import json
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import compare_candidate_sources as compare  # noqa: E402


class CompareCandidateSourcesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.baseline_path = root / "baseline.json"
        self.candidate_path = root / "candidate.json"
        self.baseline = self.report("base")
        self.candidate = self.report("candidate")
        self.write()

    def report(self, candidate_id: str, workspace: str = "source") -> dict:
        return {
            "schema_version": 1, "status": "valid", "errors": [], "candidate_id": candidate_id,
            "workspace_root": workspace, "request": {"mode": "locked-offline", "features": [],
                "all_features": False, "no_default_features": False, "filter_platform": None},
            "package_count": 2, "source_counts": {"path": 1, "registry-vendor": 1},
            "resolve_graph_sha256": "a" * 64,
            "packages": [
                {"id": "path+base#app@1.0", "name": "app", "version": "1.0.0", "source": "path",
                 "source_kind": "path", "manifest_path": f"{workspace}/apps/app/Cargo.toml"},
                {"id": "registry+index#dep@1.0", "name": "dep", "version": "1.0.0",
                 "source": "registry+https://example.invalid/index", "source_kind": "registry-vendor",
                 "manifest_path": f"{workspace}/vendor/dep/Cargo.toml"},
            ],
        }

    def enable_graph(self, report: dict) -> None:
        workspace = Path(report["workspace_root"])
        packages = report["packages"]
        identities = {}
        for package in packages:
            identity = {
                "name": package["name"], "version": package["version"],
                "source_kind": package["source_kind"],
                "source": "path" if package["source_kind"] == "path" else package["source"],
                "workspace_path": (Path(package["manifest_path"]).relative_to(workspace).parent.as_posix()
                                   if package["source_kind"] == "path" else ""),
            }
            package["identity"] = identity
            identities[package["name"]] = identity
        report["schema_version"] = 2
        report["resolved_graph"] = {"schema_version": 1, "nodes": [
            {"package": identities["app"],
             "dependencies": [{"name": "dep", "package": identities["dep"],
                               "kinds": [{"kind": None, "target": None}]}],
             "features": ["default"]},
            {"package": identities["dep"], "dependencies": [], "features": []},
        ]}
        graph_bytes = json.dumps(report["resolved_graph"]["nodes"], sort_keys=True,
                                 separators=(",", ":")).encode()
        report["resolve_graph_sha256"] = hashlib.sha256(graph_bytes).hexdigest()

    def refresh_graph_digest(self, report: dict) -> None:
        graph_bytes = json.dumps(report["resolved_graph"]["nodes"], sort_keys=True,
                                 separators=(",", ":")).encode()
        report["resolve_graph_sha256"] = hashlib.sha256(graph_bytes).hexdigest()

    def write(self) -> None:
        self.baseline_path.write_text(json.dumps(self.baseline), encoding="utf-8")
        self.candidate_path.write_text(json.dumps(self.candidate), encoding="utf-8")

    def load(self, path: Path) -> dict:
        return compare._read_report(path)

    def test_equal_inventory_ignores_candidate_workspace_root_in_package_ids(self) -> None:
        self.candidate["packages"][0]["id"] = "path+candidate#app@1.0"
        self.write()
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        self.assertTrue(result["inventory_equal"])
        self.assertEqual(result["added"], [])
        self.assertEqual(result["removed"], [])
        self.assertIn("not dependency-graph equivalence proof", result["resolve_graph_sha256"]["interpretation"])
        self.assertEqual(result["graph_comparison"]["status"], "unavailable")

    def test_v1_reports_remain_inventory_only(self) -> None:
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        self.assertEqual(result["graph_comparison"]["status"], "unavailable")
        self.assertIsNone(result["graph_comparison"]["equal"])

    def test_boolean_report_schema_cannot_downgrade_graph_validation(self) -> None:
        self.enable_graph(self.baseline)
        self.baseline["schema_version"] = True
        self.baseline["resolved_graph"] = {"schema_version": True, "nodes": []}
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "schema version 1 or 2"):
            self.load(self.baseline_path)

    def test_boolean_graph_schema_version_is_rejected(self) -> None:
        self.enable_graph(self.baseline)
        self.baseline["resolved_graph"]["schema_version"] = True
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "malformed resolved_graph"):
            self.load(self.baseline_path)

    def test_schema_v2_requires_normalized_package_identity(self) -> None:
        self.enable_graph(self.baseline)
        self.baseline["packages"][0].pop("identity")
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "missing normalized identity"):
            self.load(self.baseline_path)

    def test_boolean_package_count_is_rejected(self) -> None:
        self.baseline["packages"] = self.baseline["packages"][:1]
        self.baseline["package_count"] = True
        self.baseline["source_counts"] = {"path": 1}
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "invalid package inventory"):
            self.load(self.baseline_path)

    def test_boolean_package_id_is_rejected(self) -> None:
        self.baseline["packages"][0]["id"] = True
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "invalid package ID"):
            self.load(self.baseline_path)

    def test_path_package_source_must_match_source_kind(self) -> None:
        self.baseline["packages"][0]["source"] = "registry+https://example.invalid/index"
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "inconsistent path package source"):
            self.load(self.baseline_path)

    def test_unsafe_candidate_id_is_rejected(self) -> None:
        for candidate_id in ("../candidate", "candidate..copy", "bad/id", "x" * 65):
            with self.subTest(candidate_id=candidate_id):
                self.baseline["candidate_id"] = candidate_id
                self.write()
                with self.assertRaisesRegex(compare.ComparisonError, "invalid candidate/workspace identity"):
                    self.load(self.baseline_path)

    def test_whitespace_only_package_name_or_version_is_rejected(self) -> None:
        for field in ("name", "version"):
            with self.subTest(field=field):
                self.baseline = self.report("base")
                self.baseline["packages"][0][field] = "   "
                self.write()
                with self.assertRaisesRegex(compare.ComparisonError, "missing identity fields"):
                    self.load(self.baseline_path)

    def test_duplicate_cargo_package_ids_are_rejected(self) -> None:
        self.baseline["packages"][1]["id"] = self.baseline["packages"][0]["id"]
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "duplicate Cargo package ID"):
            self.load(self.baseline_path)

    def test_boolean_source_count_is_rejected(self) -> None:
        self.baseline["source_counts"]["path"] = True
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "source_counts"):
            self.load(self.baseline_path)

    def test_mixed_report_versions_remain_inventory_only(self) -> None:
        self.enable_graph(self.baseline)
        self.write()
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        self.assertEqual(result["graph_comparison"]["status"], "unavailable")
        self.assertIsNone(result["graph_comparison"]["equal"])

    def test_normalized_edges_and_features_ignore_candidate_roots(self) -> None:
        self.baseline = self.report("same", "baseline/source")
        self.candidate = self.report("same", "candidate/source")
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        self.write()
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        self.assertEqual(result["graph_comparison"]["status"], "available")
        self.assertTrue(result["graph_comparison"]["equal"])

    def test_normalized_graph_digest_ignores_node_order(self) -> None:
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        self.candidate["resolved_graph"]["nodes"].reverse()
        self.write()

        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))

        self.assertTrue(result["resolve_graph_sha256"]["equal"])
        self.assertTrue(result["graph_comparison"]["equal"])

    def test_detects_resolved_edge_and_enabled_feature_changes(self) -> None:
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        app = next(node for node in self.candidate["resolved_graph"]["nodes"]
                   if node["package"]["name"] == "app")
        app["dependencies"] = []
        app["features"].append("serde")
        self.refresh_graph_digest(self.candidate)
        self.write()
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        graph = result["graph_comparison"]
        self.assertFalse(graph["equal"])
        self.assertEqual(len(graph["removed_edges"]), 1)
        self.assertEqual(graph["feature_changes"][0]["added"], ["serde"])

    def test_kind_only_dependency_change_is_reported(self) -> None:
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        app = next(node for node in self.candidate["resolved_graph"]["nodes"]
                   if node["package"]["name"] == "app")
        app["dependencies"][0]["kinds"][0]["kind"] = "dev"
        self.refresh_graph_digest(self.candidate)
        self.write()

        result = compare.compare_reports(self.load(self.baseline_path),
                                         self.load(self.candidate_path))
        graph = result["graph_comparison"]
        self.assertFalse(graph["equal"])
        self.assertEqual(graph["added_edges"][0]["kinds"], [{"kind": "dev", "target": None}])
        self.assertEqual(graph["removed_edges"][0]["kinds"], [{"kind": None, "target": None}])

    def test_target_only_dependency_change_is_reported(self) -> None:
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        app = next(node for node in self.candidate["resolved_graph"]["nodes"]
                   if node["package"]["name"] == "app")
        app["dependencies"][0]["kinds"][0]["target"] = "cfg(unix)"
        self.refresh_graph_digest(self.candidate)
        self.write()

        result = compare.compare_reports(self.load(self.baseline_path),
                                         self.load(self.candidate_path))
        graph = result["graph_comparison"]
        self.assertFalse(graph["equal"])
        self.assertEqual(graph["added_edges"][0]["kinds"], [{"kind": None, "target": "cfg(unix)"}])
        self.assertEqual(graph["removed_edges"][0]["kinds"], [{"kind": None, "target": None}])

    def test_duplicate_dependency_kind_rows_are_not_erased(self) -> None:
        self.enable_graph(self.baseline)
        self.enable_graph(self.candidate)
        app = next(node for node in self.candidate["resolved_graph"]["nodes"]
                   if node["package"]["name"] == "app")
        app["dependencies"][0]["kinds"].append({"kind": None, "target": None})
        self.refresh_graph_digest(self.candidate)
        self.write()

        result = compare.compare_reports(self.load(self.baseline_path),
                                         self.load(self.candidate_path))
        graph = result["graph_comparison"]
        self.assertFalse(graph["equal"])
        self.assertEqual(graph["added_edges"][0]["kinds"], [
            {"kind": None, "target": None},
            {"kind": None, "target": None},
        ])
        self.assertEqual(graph["removed_edges"][0]["kinds"], [
            {"kind": None, "target": None},
        ])

    def test_rejects_graph_nodes_that_do_not_cover_package_inventory(self) -> None:
        self.enable_graph(self.baseline)
        self.baseline["resolved_graph"]["nodes"].pop()
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "do not match package inventory"):
            self.load(self.baseline_path)

    def test_rejects_graph_digest_mismatch(self) -> None:
        self.enable_graph(self.baseline)
        self.baseline["resolve_graph_sha256"] = "0" * 64
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "digest does not match"):
            self.load(self.baseline_path)

    def test_rejects_dependency_edge_without_kind_metadata(self) -> None:
        self.enable_graph(self.baseline)
        app = next(
            node for node in self.baseline["resolved_graph"]["nodes"]
            if node["package"]["name"] == "app"
        )
        app["dependencies"][0]["kinds"] = []
        self.refresh_graph_digest(self.baseline)
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "invalid normalized dependency edge reference"):
            self.load(self.baseline_path)

    def test_rejects_blank_dependency_kind_and_target_metadata(self) -> None:
        for field in ("kind", "target"):
            for value in ("", "   "):
                with self.subTest(field=field, value=value):
                    report = self.report("base")
                    self.enable_graph(report)
                    app = next(
                        node for node in report["resolved_graph"]["nodes"]
                        if node["package"]["name"] == "app"
                    )
                    app["dependencies"][0]["kinds"][0][field] = value
                    self.refresh_graph_digest(report)
                    self.baseline = report
                    self.write()
                    with self.assertRaisesRegex(compare.ComparisonError, "invalid dependency kind metadata"):
                        self.load(self.baseline_path)

    def test_rejects_dependency_kinds_outside_cargo_metadata_contract(self) -> None:
        for value in ("normal", "runtime", "unknown", True):
            with self.subTest(value=value):
                report = self.report("base")
                self.enable_graph(report)
                app = next(
                    node for node in report["resolved_graph"]["nodes"]
                    if node["package"]["name"] == "app"
                )
                app["dependencies"][0]["kinds"][0]["kind"] = value
                self.refresh_graph_digest(report)
                self.baseline = report
                self.write()
                with self.assertRaisesRegex(compare.ComparisonError, "invalid dependency kind metadata"):
                    self.load(self.baseline_path)

    def test_rejects_blank_dependency_alias(self) -> None:
        for alias in ("", " \t "):
            with self.subTest(alias=alias):
                report = self.report("base")
                self.enable_graph(report)
                app = next(
                    node for node in report["resolved_graph"]["nodes"]
                    if node["package"]["name"] == "app"
                )
                app["dependencies"][0]["name"] = alias
                self.refresh_graph_digest(report)
                self.baseline = report
                self.write()
                with self.assertRaisesRegex(
                    compare.ComparisonError,
                    "invalid normalized dependency edge reference",
                ):
                    self.load(self.baseline_path)

    def test_rejects_blank_enabled_feature(self) -> None:
        for feature in ("", " \t "):
            with self.subTest(feature=feature):
                report = self.report("base")
                self.enable_graph(report)
                app = next(
                    node for node in report["resolved_graph"]["nodes"]
                    if node["package"]["name"] == "app"
                )
                app["features"] = [feature]
                self.refresh_graph_digest(report)
                self.baseline = report
                self.write()
                with self.assertRaisesRegex(compare.ComparisonError, "invalid or duplicate enabled feature"):
                    self.load(self.baseline_path)

    def test_reports_added_and_removed_package_source_identities(self) -> None:
        self.candidate["packages"][1]["version"] = "2.0.0"
        self.candidate["packages"][1]["id"] = "registry+index#dep@2.0"
        self.write()
        result = compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))
        self.assertFalse(result["inventory_equal"])
        self.assertEqual([item["version"] for item in result["added"]], ["2.0.0"])
        self.assertEqual([item["version"] for item in result["removed"]], ["1.0.0"])

    def test_rejects_mismatched_feature_selection(self) -> None:
        self.candidate["request"]["features"] = ["optional"]
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "selections must match"):
            compare.compare_reports(self.load(self.baseline_path), self.load(self.candidate_path))

    def test_feature_selection_order_does_not_change_selection_identity(self) -> None:
        self.baseline["request"]["features"] = ["first", "second"]
        self.candidate["request"]["features"] = ["second", "first"]
        self.write()

        result = compare.compare_reports(
            self.load(self.baseline_path), self.load(self.candidate_path)
        )

        self.assertTrue(result["selection_match"])

    def test_feature_selector_separators_and_empty_tokens_are_normalized(self) -> None:
        self.baseline["request"]["features"] = ["first, second"]
        self.candidate["request"]["features"] = ["second", "first"]
        self.write()
        result = compare.compare_reports(
            self.load(self.baseline_path), self.load(self.candidate_path)
        )
        self.assertTrue(result["selection_match"])

        self.baseline["request"]["features"] = []
        self.candidate["request"]["features"] = ["   ", ","]
        self.write()
        result = compare.compare_reports(
            self.load(self.baseline_path), self.load(self.candidate_path)
        )
        self.assertTrue(result["selection_match"])

    def test_rejects_invalid_or_tampered_report(self) -> None:
        self.candidate["source_counts"]["path"] = 99
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "source_counts"):
            self.load(self.candidate_path)

    def test_rejects_path_package_outside_reported_workspace(self) -> None:
        self.candidate["packages"][0]["manifest_path"] = "other/apps/app/Cargo.toml"
        self.write()
        with self.assertRaisesRegex(compare.ComparisonError, "outside its workspace"):
            self.load(self.candidate_path)

    def test_rejects_nul_in_workspace_and_package_paths(self) -> None:
        cases = (
            ("workspace_root", "source\x00root"),
            ("manifest_path", "source/apps/bad\x00name/Cargo.toml"),
        )
        for field, value in cases:
            with self.subTest(field=field):
                self.candidate = self.report("candidate")
                if field == "workspace_root":
                    self.candidate["workspace_root"] = value
                    for package in self.candidate["packages"]:
                        package["manifest_path"] = value + package["manifest_path"][len("source"):]
                else:
                    self.candidate["packages"][0]["manifest_path"] = value
                self.write()
                with self.assertRaisesRegex(compare.ComparisonError, "unsafe workspace_root|unsafe manifest_path"):
                    self.load(self.candidate_path)


if __name__ == "__main__":
    unittest.main()
