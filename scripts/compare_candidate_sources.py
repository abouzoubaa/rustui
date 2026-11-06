#!/usr/bin/env python3
"""Compare package inventories and, when available, normalized P3H graphs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
from typing import Any

try:
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts
except ModuleNotFoundError:
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from p3_candidate_fs import CandidateFSError, PinnedDir, relative_parts


SCHEMA_VERSION = 2
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
IDENTIFIER_RE = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}\Z")
IDENTITY_FIELDS = {"name", "version", "source_kind", "source", "workspace_path"}
SCOPE = (
    "read-only comparison of P3H package inventories and normalized dependency edges/features when both inputs carry graph data; "
    "no provenance approval build correctness GUI performance or promotion"
)


class ComparisonError(ValueError):
    """One or both Cargo source reports do not satisfy the comparison contract."""


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ComparisonError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> Any:
    raise ComparisonError(f"nonstandard JSON constant: {value}")


def _normalized_graph_bytes(nodes: list[dict[str, Any]]) -> bytes:
    """Serialize graph data independently of JSON array ordering."""
    normalized = []
    for node in nodes:
        dependencies = []
        for edge in node["dependencies"]:
            dependencies.append({
                "name": edge["name"],
                "package": edge["package"],
                "kinds": sorted(edge["kinds"], key=lambda item: json.dumps(item, sort_keys=True)),
            })
        normalized.append({
            "package": node["package"],
            "dependencies": sorted(dependencies, key=lambda item: json.dumps(item, sort_keys=True)),
            "features": sorted(node["features"]),
        })
    normalized.sort(key=lambda item: json.dumps(item["package"], sort_keys=True))
    return json.dumps(normalized, sort_keys=True, separators=(",", ":")).encode()


def _identity_key(identity: Any, report_path: Path) -> tuple[str, ...]:
    if not isinstance(identity, dict) or set(identity) != IDENTITY_FIELDS:
        raise ComparisonError(f"report {report_path} has malformed normalized package identity")
    name, version, kind, source, workspace_path = (identity.get(field) for field in
                                                    ("name", "version", "source_kind", "source", "workspace_path"))
    if not all(isinstance(value, str) and value for value in (name, version, kind, source)):
        raise ComparisonError(f"report {report_path} has incomplete normalized package identity")
    if not isinstance(workspace_path, str):
        raise ComparisonError(f"report {report_path} has invalid normalized workspace path")
    if kind == "path":
        path = PurePosixPath(workspace_path)
        if path.is_absolute() or path.as_posix() != workspace_path or any(part in ("", "..") for part in path.parts):
            raise ComparisonError(f"report {report_path} has unsafe normalized path package identity")
        if source != "path":
            raise ComparisonError(f"report {report_path} has inconsistent path package identity")
    elif kind == "registry-vendor":
        if not source.startswith("registry+") or workspace_path:
            raise ComparisonError(f"report {report_path} has inconsistent registry package identity")
    else:
        raise ComparisonError(f"report {report_path} has unsupported normalized source kind: {kind}")
    return (name, version, kind, source, workspace_path)


def _read_graph(data: dict[str, Any], package_keys: set[tuple[str, ...]], path: Path) -> dict:
    graph = data.get("resolved_graph")
    if (not isinstance(graph, dict)
            or type(graph.get("schema_version")) is not int
            or graph["schema_version"] != 1
            or not isinstance(graph.get("nodes"), list)):
        raise ComparisonError(f"report {path} has malformed resolved_graph")
    nodes: dict[tuple[str, ...], dict[str, Any]] = {}
    for index, node in enumerate(graph["nodes"]):
        if not isinstance(node, dict) or set(node) != {"package", "dependencies", "features"}:
            raise ComparisonError(f"report {path} resolved_graph.nodes[{index}] is malformed")
        package_key = _identity_key(node["package"], path)
        if package_key not in package_keys or package_key in nodes:
            raise ComparisonError(f"report {path} resolved graph package identity is missing or duplicated")
        dependencies = node["dependencies"]
        features = node["features"]
        if not isinstance(dependencies, list) or not isinstance(features, list):
            raise ComparisonError(f"report {path} graph dependencies/features must be arrays")
        feature_set: set[str] = set()
        for feature in features:
            if not isinstance(feature, str) or not feature.strip() or feature in feature_set:
                raise ComparisonError(f"report {path} has invalid or duplicate enabled feature")
            feature_set.add(feature)
        edges: set[tuple[Any, ...]] = set()
        for edge in dependencies:
            if not isinstance(edge, dict) or set(edge) != {"name", "package", "kinds"}:
                raise ComparisonError(f"report {path} has a malformed normalized dependency edge")
            alias, kinds = edge["name"], edge["kinds"]
            target = _identity_key(edge["package"], path)
            if (
                not isinstance(alias, str)
                or not alias.strip()
                or target not in package_keys
                or not isinstance(kinds, list)
                or not kinds
            ):
                raise ComparisonError(f"report {path} has an invalid normalized dependency edge reference")
            normalized_kinds: list[tuple[str | None, str | None]] = []
            for kind in kinds:
                if not isinstance(kind, dict) or set(kind) != {"kind", "target"}:
                    raise ComparisonError(f"report {path} has malformed dependency kind metadata")
                dep_kind, dep_target = kind["kind"], kind["target"]
                if (
                    dep_kind is not None
                    and (not isinstance(dep_kind, str) or dep_kind not in {"build", "dev"})
                ) or (
                    dep_target is not None
                    and (not isinstance(dep_target, str) or not dep_target.strip())
                ):
                    raise ComparisonError(f"report {path} has invalid dependency kind metadata")
                normalized_kinds.append((dep_kind, dep_target))
            edge_key = (alias, target, tuple(sorted(normalized_kinds, key=repr)))
            if edge_key in edges:
                raise ComparisonError(f"report {path} has duplicate normalized dependency edge")
            edges.add(edge_key)
        nodes[package_key] = {"dependencies": edges, "features": feature_set}
    if set(nodes) != package_keys:
        raise ComparisonError(f"report {path} resolved graph nodes do not match package inventory")
    graph_bytes = _normalized_graph_bytes(graph["nodes"])
    if hashlib.sha256(graph_bytes).hexdigest() != data.get("resolve_graph_sha256"):
        raise ComparisonError(f"report {path} normalized graph digest does not match its graph data")
    return nodes


def _read_report(path: Path, pinned_root: PinnedDir | None = None) -> dict[str, Any]:
    try:
        if pinned_root is None:
            # Standalone CLI: the user-selected report directory is the trust root.
            with PinnedDir.open_root(path.parent.resolve(strict=True)) as report_root:
                raw = report_root.read_bytes(path.name, limit=32 * 1024 * 1024)
        else:
            relative = Path(path)
            if relative.is_absolute():
                raise ComparisonError("inherited repository root requires report paths relative to that root")
            relative_parts(relative.as_posix())
            raw = pinned_root.read_bytes(relative.as_posix(), limit=32 * 1024 * 1024)
        data = json.loads(raw.decode("utf-8"), object_pairs_hook=_unique_object,
                          parse_constant=_reject_constant)
    except (OSError, UnicodeError, json.JSONDecodeError, CandidateFSError) as error:
        raise ComparisonError(f"cannot read report {path}: {error}") from error
    report_version = data.get("schema_version") if isinstance(data, dict) else None
    if (not isinstance(data, dict) or type(report_version) is not int
            or report_version not in (1, 2) or data.get("status") != "valid"):
        raise ComparisonError(f"report {path} must be a valid P3H schema version 1 or 2 report")
    request = data.get("request")
    if (not isinstance(request, dict)
            or set(request) != {"mode", "features", "all_features", "no_default_features", "filter_platform"}
            or request.get("mode") != "locked-offline"
            or not isinstance(request.get("features"), list)
            or any(not isinstance(feature, str) for feature in request["features"])
            or not isinstance(request.get("all_features"), bool)
            or not isinstance(request.get("no_default_features"), bool)
            or (request.get("filter_platform") is not None and not isinstance(request.get("filter_platform"), str))):
        raise ComparisonError(f"report {path} is not a locked-offline P3H report")
    if data.get("errors") != []:
        raise ComparisonError(f"report {path} must have an empty errors array")
    candidate_id, workspace = data.get("candidate_id"), data.get("workspace_root")
    if (not isinstance(candidate_id, str) or IDENTIFIER_RE.fullmatch(candidate_id) is None
            or ".." in candidate_id or not isinstance(workspace, str)):
        raise ComparisonError(f"report {path} has invalid candidate/workspace identity")
    workspace_path = PurePosixPath(workspace)
    if ("\x00" in workspace or workspace_path.is_absolute() or not workspace_path.parts
            or workspace_path.as_posix() != workspace
            or any(part in ("", ".", "..") for part in workspace_path.parts)):
        raise ComparisonError(f"report {path} has an unsafe workspace_root")
    graph_hash = data.get("resolve_graph_sha256")
    if not isinstance(graph_hash, str) or SHA256_RE.fullmatch(graph_hash) is None:
        raise ComparisonError(f"report {path} has an invalid resolve_graph_sha256")
    packages = data.get("packages")
    package_count = data.get("package_count")
    if (not isinstance(packages, list) or not packages
            or type(package_count) is not int or package_count != len(packages)):
        raise ComparisonError(f"report {path} has an invalid package inventory")
    normalized: dict[tuple[str, ...], dict[str, str]] = {}
    counts: dict[str, int] = {}
    package_ids: set[str] = set()
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            raise ComparisonError(f"report {path} packages[{index}] must be an object")
        package_id = package.get("id")
        if not isinstance(package_id, str) or not package_id.strip():
            raise ComparisonError(f"report {path} packages[{index}] has an invalid package ID")
        if package_id in package_ids:
            raise ComparisonError(f"report {path} has duplicate Cargo package ID: {package_id}")
        package_ids.add(package_id)
        name, version = package.get("name"), package.get("version")
        kind, source, manifest = (package.get(field) for field in ("source_kind", "source", "manifest_path"))
        if not all(isinstance(value, str) and value.strip()
                   for value in (name, version, kind, source, manifest)):
            raise ComparisonError(f"report {path} packages[{index}] has missing identity fields")
        manifest_path = PurePosixPath(manifest)
        if ("\x00" in manifest or manifest_path.is_absolute()
                or manifest_path.as_posix() != manifest
                or manifest_path.name != "Cargo.toml"
                or any(part in ("", ".", "..") for part in manifest_path.parts)):
            raise ComparisonError(f"report {path} packages[{index}] has an unsafe manifest_path")
        if kind == "path":
            if source != "path":
                raise ComparisonError(f"report {path} has inconsistent path package source")
            try:
                location = manifest_path.relative_to(workspace_path).parent.as_posix()
            except ValueError as error:
                raise ComparisonError(f"report {path} path package is outside its workspace") from error
            identity = {"name": name, "version": version, "source_kind": kind, "source": "path",
                        "workspace_path": location}
        elif kind == "registry-vendor" and source.startswith("registry+"):
            try:
                manifest_path.relative_to(workspace_path / "vendor")
            except ValueError as error:
                raise ComparisonError(f"report {path} registry package is outside workspace vendor/") from error
            identity = {"name": name, "version": version, "source_kind": kind, "source": source,
                        "workspace_path": ""}
        else:
            raise ComparisonError(f"report {path} contains unsupported package source kind: {kind}")
        if report_version == 2 and "identity" not in package:
            raise ComparisonError(f"report {path} packages[{index}] missing normalized identity")
        key = _identity_key(package.get("identity") if report_version == 2 else identity, path)
        derived_key = _identity_key(identity, path)
        if key != derived_key:
            raise ComparisonError(f"report {path} package identity does not match its manifest/source fields")
        if key in normalized:
            raise ComparisonError(f"report {path} has duplicate normalized package identity: {key}")
        normalized[key] = {"name": name, "version": version, "source_kind": kind,
                           "source": identity["source"], "workspace_path": identity["workspace_path"]}
        counts[kind] = counts.get(kind, 0) + 1
    source_counts = data.get("source_counts")
    if (not isinstance(source_counts, dict)
            or any(type(count) is not int for count in source_counts.values())
            or source_counts != dict(sorted(counts.items()))):
        raise ComparisonError(f"report {path} source_counts do not match packages")
    graph = _read_graph(data, set(normalized), path) if report_version == 2 else None
    if report_version == 2 and graph is None:
        raise ComparisonError(f"report {path} schema 2 requires normalized resolved_graph")
    return {"data": data, "packages": normalized, "graph": graph}


def _display_identity(key: tuple[str, ...], table: dict[tuple[str, ...], dict[str, str]]) -> dict[str, str]:
    return table[key]


def _edge_set(graph: dict[tuple[str, ...], dict[str, Any]]) -> set[tuple[Any, ...]]:
    return {(source, *dependency) for source, node in graph.items()
            for dependency in node["dependencies"]}


def compare_reports(baseline: dict[str, Any], candidate: dict[str, Any]) -> dict[str, Any]:
    """Compare two decoded and validated P3H reports."""
    base, cand = baseline["data"], candidate["data"]
    base_request, candidate_request = base["request"], cand["request"]

    def selected_features(request: dict[str, Any]) -> list[str]:
        return sorted({
            feature
            for selector in request["features"]
            for feature in re.split(r"[\s,]+", selector)
            if feature
        })

    if (
        {**base_request, "features": selected_features(base_request)}
        != {**candidate_request, "features": selected_features(candidate_request)}
    ):
        raise ComparisonError("baseline and candidate Cargo metadata selections must match")
    base_packages, candidate_packages = baseline["packages"], candidate["packages"]
    added_keys = sorted(candidate_packages.keys() - base_packages.keys())
    removed_keys = sorted(base_packages.keys() - candidate_packages.keys())
    result = {
        "schema_version": SCHEMA_VERSION,
        "status": "valid",
        "scope": SCOPE,
        "baseline_candidate_id": base["candidate_id"],
        "candidate_id": cand["candidate_id"],
        "selection_match": True,
        "baseline_report_schema": base["schema_version"],
        "candidate_report_schema": cand["schema_version"],
        "baseline_package_count": len(base_packages),
        "candidate_package_count": len(candidate_packages),
        "inventory_equal": not added_keys and not removed_keys,
        "added": [_display_identity(key, candidate_packages) for key in added_keys],
        "removed": [_display_identity(key, base_packages) for key in removed_keys],
        "resolve_graph_sha256": {
            "baseline": base["resolve_graph_sha256"],
            "candidate": cand["resolve_graph_sha256"],
            "equal": base["resolve_graph_sha256"] == cand["resolve_graph_sha256"],
            "interpretation": "normalized graph digest" if base["schema_version"] == cand["schema_version"] == 2
            else "legacy opaque digest; equality is not dependency-graph equivalence proof",
        },
        "graph_comparison": {"status": "unavailable", "equal": None,
                              "reason": "both reports must carry normalized graph data; P3H schema v1 is inventory-only"},
        "errors": [],
    }
    base_graph, candidate_graph = baseline["graph"], candidate["graph"]
    if base_graph is None or candidate_graph is None:
        return result

    base_edges, candidate_edges = _edge_set(base_graph), _edge_set(candidate_graph)

    def edge_record(edge: tuple[Any, ...], packages: dict[tuple[str, ...], dict[str, str]]) -> dict[str, Any]:
        source_key, alias, dependency_key, kinds = edge
        return {"from": packages.get(source_key, base_packages.get(source_key)),
                "name": alias,
                "package": packages.get(dependency_key, candidate_packages.get(dependency_key)),
                "kinds": [{"kind": kind, "target": target} for kind, target in kinds]}

    added_edges = sorted(candidate_edges - base_edges, key=repr)
    removed_edges = sorted(base_edges - candidate_edges, key=repr)
    common_keys = base_graph.keys() & candidate_graph.keys()
    feature_changes = []
    for key in sorted(common_keys):
        before, after = base_graph[key]["features"], candidate_graph[key]["features"]
        if before != after:
            feature_changes.append({"package": base_packages[key], "baseline": sorted(before),
                                    "candidate": sorted(after), "added": sorted(after - before),
                                    "removed": sorted(before - after)})
    graph_equal = (not added_keys and not removed_keys and not added_edges and not removed_edges
                   and not feature_changes)
    result["graph_comparison"] = {
        "status": "available",
        "equal": graph_equal,
        "added_edges": [edge_record(edge, candidate_packages) for edge in added_edges],
        "removed_edges": [edge_record(edge, base_packages) for edge in removed_edges],
        "feature_changes": feature_changes,
    }
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline_report", type=Path)
    parser.add_argument("candidate_report", type=Path)
    args = parser.parse_args()
    try:
        inherited_fd = os.environ.get("RUSTUI_P3_REPO_ROOT_FD")
        if inherited_fd is None:
            baseline = _read_report(args.baseline_report)
            candidate = _read_report(args.candidate_report)
        else:
            with PinnedDir.from_fd(int(inherited_fd)) as repo_root:
                baseline = _read_report(args.baseline_report, repo_root)
                candidate = _read_report(args.candidate_report, repo_root)
        report = compare_reports(baseline, candidate)
    except ComparisonError as error:
        report = {"schema_version": SCHEMA_VERSION, "status": "invalid", "scope": SCOPE,
                  "errors": [str(error)]}
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] == "valid" else 1


if __name__ == "__main__":
    raise SystemExit(main())
