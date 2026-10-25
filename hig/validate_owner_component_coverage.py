#!/usr/bin/env python3
"""Static, HIG-local checks for the owner-supplied macOS component matrix."""

from __future__ import annotations

import csv
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parent
OWNER = ROOT / "OWNER_COMPONENT_REFERENCE_2026-10-22.md"
MATRIX = ROOT / "MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv"
CLAIM_MATRIX = ROOT / "MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv"
BASELINE = ROOT / "macos27.production-baseline.2026-10-22.json"
THEME = ROOT / "macos27.json"
CANDIDATE = ROOT / "macos27.design-v2-candidate.json"


def named_rows() -> list[tuple[str, str]]:
    source = OWNER.read_text(encoding="utf-8")
    inventory = source.split("## Named-item inventory", 1)[1].split(
        "## Appearance requirements", 1
    )[0]
    section = ""
    rows: list[tuple[str, str]] = []
    for line in inventory.splitlines():
        if line.startswith("### "):
            section = line[4:].strip()
        elif section and line.startswith("- "):
            rows.append((section, line[2:]))
    return rows


def supplemental_rows() -> list[tuple[str, str, str]]:
    source = OWNER.read_text(encoding="utf-8")
    ranges = (
        ("A", "Appearance requirements", "## Appearance requirements and design claims", "## Version-sensitive claim register"),
        ("V", "Version-sensitive claim register", "## Version-sensitive claim register", "## User-supplied source links"),
        ("L", "User-supplied source links", "## User-supplied source links", "## Reconciliation checklist"),
    )
    rows: list[tuple[str, str, str]] = []
    for prefix, heading, start, end in ranges:
        block = source.split(start, 1)[1].split(end, 1)[0]
        labels = [line[2:] for line in block.splitlines() if line.startswith("- ")]
        rows.extend((f"{prefix}{index:02d}", heading, label) for index, label in enumerate(labels, 1))
    return rows


def main() -> int:
    failures: list[str] = []
    owner_items = named_rows()
    with MATRIX.open(newline="", encoding="utf-8") as stream:
        matrix = list(csv.DictReader(stream))
    expected_supplemental = supplemental_rows()
    with CLAIM_MATRIX.open(newline="", encoding="utf-8") as stream:
        supplemental = list(csv.DictReader(stream))

    if len(owner_items) != 148:
        failures.append(f"owner inventory rows: expected 148, found {len(owner_items)}")
    if len({section for section, _ in owner_items}) != 16:
        failures.append("owner inventory does not contain exactly 16 named sections")
    if len(matrix) != len(owner_items):
        failures.append(f"matrix rows: expected {len(owner_items)}, found {len(matrix)}")
    for index, ((section, label), row) in enumerate(zip(owner_items, matrix), start=1):
        if row.get("source_row") != f"O{index:03d}":
            failures.append(f"row {index}: source_row order/id mismatch")
        if row.get("source_section") != section:
            failures.append(f"row {index}: section mismatch")
        if row.get("exact_source_label") != label:
            failures.append(f"row {index}: label differs from owner source")
        for field in (
            "canonical_concept_id",
            "api_hints_verbatim",
            "aliases_and_cross_references",
            "category_and_ownership",
            "apple_claim_status",
            "official_apple_evidence",
            "theme_json_status",
            "rustui_code_runtime_status",
            "design_candidate_status",
            "proposed_contract",
            "accessibility_input_localization",
            "native_fallback_or_exclusion",
            "validation_plan",
        ):
            if not row.get(field, "").strip():
                failures.append(f"row {index}: empty required field {field}")
        for target in re.findall(r"O\d{3}", row.get("aliases_and_cross_references", "")):
            if not 1 <= int(target[1:]) <= len(owner_items):
                failures.append(f"row {index}: cross-reference {target} is out of range")
        for concept in row.get("related_theme_ids", "").split(";"):
            theme_id = concept.strip().split(" ", 1)[0]
            if theme_id != "none" and theme_id not in json.loads(THEME.read_text(encoding="utf-8"))["components"]:
                failures.append(f"row {index}: unknown related theme ID {theme_id}")
        for url in re.findall(r"https?://[^\s;]+", row.get("official_apple_evidence", "")):
            if urlparse(url).hostname != "developer.apple.com":
                failures.append(f"row {index}: non-Apple official evidence URL {url}")

    if len({row.get("source_row") for row in matrix}) != len(matrix):
        failures.append("source_row IDs are not unique")
    if len(supplemental) != len(expected_supplemental) or len(supplemental) != 28:
        failures.append(f"supplemental rows: expected 28, found {len(supplemental)}")
    supplemental_counts = {
        prefix: sum(source_id.startswith(prefix) for source_id, _, _ in expected_supplemental)
        for prefix in ("A", "V", "L")
    }
    if supplemental_counts != {"A": 8, "V": 11, "L": 9}:
        failures.append(f"supplemental source item counts changed: {supplemental_counts}")
    for index, ((source_id, section, label), row) in enumerate(zip(expected_supplemental, supplemental), 1):
        if row.get("source_id") != source_id or row.get("source_section") != section:
            failures.append(f"supplemental row {index}: source ID/section mismatch")
        if row.get("exact_source_item") != label:
            failures.append(f"supplemental row {index}: exact source text mismatch")
        for field in (
            "canonical_claim_family",
            "apple_claim_status",
            "theme_and_runtime_reconciliation",
            "design_candidate_reconciliation",
            "fallback_or_decision",
            "official_apple_evidence_or_lead",
            "validation_plan",
        ):
            if not row.get(field, "").strip():
                failures.append(f"supplemental row {index}: empty required field {field}")
        for url in re.findall(r"https?://[^\s;]+", row.get("official_apple_evidence_or_lead", "")):
            hostname = urlparse(url).hostname
            if source_id.startswith("V") and hostname != "developer.apple.com":
                failures.append(f"supplemental claim {source_id}: evidence URL is not Apple primary source: {url}")
            if source_id.startswith("L") and hostname is None:
                failures.append(f"supplemental source {source_id}: malformed URL")
    ids = json.loads(THEME.read_text(encoding="utf-8"))["components"]
    baseline_bytes = BASELINE.read_bytes()
    theme_bytes = THEME.read_bytes()
    baseline_hash = hashlib.sha256(baseline_bytes).hexdigest()
    theme_hash = hashlib.sha256(theme_bytes).hexdigest()
    if theme_hash != baseline_hash:
        failures.append("HIG baseline JSON differs from preserved production snapshot")
    rule_count = sum(len(record.get("styles", [])) for record in ids.values())
    if len(ids) != 41 or rule_count != 419:
        failures.append(f"baseline inventory changed: {len(ids)} IDs / {rule_count} styles")

    candidate = json.loads(CANDIDATE.read_text(encoding="utf-8"))
    if candidate.get("runtime_compatible") is not False:
        failures.append("design candidate must remain runtime_compatible=false")
    coverage = candidate.get("owner_reference_coverage", {})
    if coverage.get("named_item_rows") != 148:
        failures.append("candidate owner_reference_coverage.named_item_rows must be 148")
    if coverage.get("matrix") != MATRIX.name:
        failures.append("candidate does not point to the local coverage matrix")
    if coverage.get("supplemental_matrix") != CLAIM_MATRIX.name or coverage.get("supplemental_rows", {}).get("total") != 28:
        failures.append("candidate does not point to the complete supplemental owner ledger")
    preserved = candidate.get("preservation", {})
    if preserved.get("v1_component_count") != 41 or preserved.get("v1_style_count") != 419:
        failures.append("candidate preservation contract changed from 41 IDs / 419 styles")
    if candidate.get("source_baselines", {}).get("production_theme", {}).get("sha256") != theme_hash:
        failures.append("candidate production-theme provenance hash differs from HIG baseline")
    expected_candidate_counts = {
        "additional_rustui_component_contracts": 31,
        "absent_reusable_patterns": 5,
        "native_integrations": 11,
        "composition_and_reference_examples": 9,
    }
    for field, expected in expected_candidate_counts.items():
        if len(candidate.get(field, [])) != expected:
            failures.append(f"candidate {field}: expected {expected}, found {len(candidate.get(field, []))}")
    if len(candidate.get("cross_cutting_owner_contracts", [])) != 8:
        failures.append("candidate cross-cutting owner contract count is not 8")
    if len(candidate.get("owner_capability_gaps", [])) != 8:
        failures.append("candidate owner capability gap count is not 8")
    fidelity = candidate.get("acceptance_criteria", {})
    if fidelity.get("path") != "MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md":
        failures.append("candidate does not point to the macOS 27 visual fidelity gate")
    elif not (ROOT / fidelity["path"]).is_file():
        failures.append("candidate visual fidelity gate file is missing")
    if not fidelity.get("status", "").startswith("NOT DEMONSTRATED"):
        failures.append("candidate must retain NOT DEMONSTRATED fidelity status")
    if fidelity.get("generated_images_are_supplementary") is not True:
        failures.append("candidate must mark generated images as supplementary")
    if fidelity.get("static_catalogue_or_json_checks_can_establish_fidelity") is not False:
        failures.append("candidate must prohibit static checks from establishing fidelity")
    if fidelity.get("macos_26_1_sdk_is_a_macOS_27_reference") is not False:
        failures.append("candidate must not treat macOS 26.1 SDK as macOS 27 evidence")

    category_counts = {}
    for row in matrix:
        category = row["category_and_ownership"]
        category_counts[category] = category_counts.get(category, 0) + 1
    expected_categories = {
        "system-owned surface; RustUI must not recreate": 17,
        "native system integration; OS owns canonical presentation": 5,
        "native host integration or composed app behavior": 51,
        "RustUI component/style contract or composition": 64,
        "native platform material/API; current RustUI adapter does not render it": 10,
        "unsupported / retired platform hardware": 1,
    }
    if category_counts != expected_categories:
        failures.append(f"owner primary ownership counts changed: {category_counts}")

    visual_map = ROOT / "MACOS27_VISUAL_REFERENCE_MAP_2026-10-22.csv"
    with visual_map.open(newline="", encoding="utf-8") as stream:
        visual_rows = list(csv.DictReader(stream))
    if len(visual_rows) != 86:
        failures.append(f"visual reference map: expected 86 rows, found {len(visual_rows)}")
    for row in visual_rows:
        image_path = (ROOT / row["stored_path"]).resolve()
        if not image_path.is_relative_to(ROOT.resolve()) or not image_path.is_file():
            failures.append(f"visual reference path missing/outside hig: {row['stored_path']}")
            continue
        if image_path.stat().st_size != int(row["size_bytes"]):
            failures.append(f"visual reference size mismatch: {row['stored_path']}")
        if hashlib.sha256(image_path.read_bytes()).hexdigest() != row["sha256"]:
            failures.append(f"visual reference hash mismatch: {row['stored_path']}")

    print(
        f"OWNER_MATRIX rows={len(matrix)}/{len(owner_items)} sections="
        f"{len({section for section, _ in owner_items})} exact_source_labels=checked"
    )
    print(
        f"BASELINE ids={len(ids)} styles={rule_count} "
        f"sha256={theme_hash} matches_production_snapshot={theme_hash == baseline_hash}"
    )
    print("CANDIDATE counts=31/5/11/9 runtime_compatible=false owner_rows=148 linked_matrix=true")
    print(f"SUPPLEMENTAL_OWNER_ROWS rows={len(supplemental)}/28 exact_source_items=checked")
    print("FIDELITY_GATE status=NOT DEMONSTRATED; static checks do not establish native fidelity")
    print(f"VISUAL_REFERENCES rows={len(visual_rows)} paths_sizes_hashes=checked")
    if failures:
        print("FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("PASS: static HIG-local owner coverage checks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
