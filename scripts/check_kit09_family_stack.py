#!/usr/bin/env python3
"""Apply, compile, prove, and reverse the GPUI Kit 0.9 candidate stack."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from typing import Any

import check_kit09_core_stack
from fixture_target_dir import fixture_target_root
from check_kit09_core_stack import apply_stack
from prepare_gpui_kit_09_fixture import (
    DEFAULT_MANIFEST,
    FixtureError,
    build_fixture,
    load_manifest,
)


ROOT = Path(__file__).resolve().parent.parent
PATCH_DIR = ROOT / "patches/candidate-kit-0.9"
OUTPUT = PATCH_DIR / "family-manifest.json"
FIXTURE_BUILD_CACHE = "kit09-fixture-build"
FIXTURE_MTIME_EPOCH = 1_600_000_000
CORE_ORDER = check_kit09_core_stack.CANDIDATE_ORDER
A3_4A_ORDER = (
    "t13-modal-focus-recovery-v1-kit09.patch",
    "gpui-component-tracked-dialog-focus-v1-kit09.patch",
    "gpui-component-sheet-close-accessibility-v1-kit09.patch",
)
A3_4B_ORDER = (
    "gpui-component-popup-menu-disabled-confirm-v1-kit08.patch",
    "r15b-menu-item-disabled-v1-kit09.patch",
    "r15b-popup-menu-recovery-v1-kit09.patch",
    "r15b-dropdown-menu-refresh-v1.patch",
    "gpui-component-select-popup-rtl-v1-kit08.patch",
    "gpui-component-select-rtl-alignment-v1.patch",
    "gpui-component-select-rtl-alignment-v2-kit08.patch",
    "gpui-component-select-rtl-alignment-v3-kit08.patch",
    "gpui-component-select-rtl-auto-width-v4-kit08.patch",
    "gpui-component-select-rtl-auto-width-v6-kit08.patch",
)
A3_4C_ORDER = (
    "gpui-selectable-list-semantics-v2.patch",
    "gpui-component-tree-accessible-name-v1-kit08.patch",
    "gpui-tree-aria-disabled-row-v1-kit08.patch",
    "gpui-tree-aria-level-v1-kit08.patch",
    "gpui-tree-disabled-keyboard-v2.patch",
    "r15-tree-disabled-focus-v1.patch",
    "gpui-component-table-delegate-accessibility-v1-kit09.patch",
)
A3_4D_ORDER = (
    "gpui-base-slider-endpoint-v1-kit09.patch",
    "gpui-base-slider-first-move-v1.patch",
    "gpui-base-slider-set-value-v2-kit09.patch",
    "gpui-base-slider-zero-track-v1.patch",
    "gpui-base-resize-first-move-v1-kit08.patch",
    "gpui-base-resize-first-move-v3-kit08.patch",
)
A3_4E_ORDER = (
    "gpui-base-calendar-date-limits-v1-kit08.patch",
    "gpui-calendar-keyboard-v1.patch",
    "gpui-calendar-rtl-v1-kit08.patch",
    "gpui-calendar-multi-month-heading-v1-kit08.patch",
    "gpui-component-calendar-fr-ar-v1-kit09.patch",
    "gpui-component-date-picker-accessibility-v1.patch",
    "gpui-component-date-picker-localization-v2-kit08.patch",
    "gpui-component-date-picker-rtl-alignment-v1-kit08.patch",
)
A3_4F_ORDER = (
    "gpui-base-input-validation-description-v1-kit09.patch",
    "gpui-input-placeholder-color-v1-kit09.patch",
)
A3_4G_ORDER = ("gpui-toolbar-rtl-v1-kit09.patch",)
A3_4H_ORDER = (
    "gpui-component-web-syntax-v1-kit09.patch",
    "gpui-component-web-syntax-runtime-v1-kit08.patch",
    "gpui-component-web-syntax-runtime-v2-kit08.patch",
    "gpui-component-web-syntax-runtime-v3-kit08.patch",
)
A3_4I_ORDER = (
    "gpui-macos-anchored-popup-v1-kit09.patch",
    "gpui-macos-popup-child-teardown-v1-kit09.patch",
)
FAMILY_ORDER = (
    A3_4A_ORDER + A3_4B_ORDER + A3_4C_ORDER + A3_4D_ORDER + A3_4E_ORDER
    + A3_4F_ORDER + A3_4G_ORDER + A3_4H_ORDER + A3_4I_ORDER
)


FAMILY_PROOF_CRATE = ROOT / "scripts/kit08_family_proofs"


def candidate_build_key() -> str:
    """Key fixture artifacts by every input that shapes the temporary workspace."""
    inputs = [
        DEFAULT_MANIFEST,
        ROOT / "scripts/check_kit09_family_stack.py",
        ROOT / "scripts/check_kit09_core_stack.py",
        ROOT / "scripts/fixture_target_dir.py",
        ROOT / "scripts/prepare_gpui_kit_08_fixture.py",
        *(path for path in PATCH_DIR.rglob("*") if path.is_file()),
        *(path for path in FAMILY_PROOF_CRATE.glob("tests/**/*") if path.is_file()),
    ]
    return input_digest(inputs)


def input_digest(paths: list[Path]) -> str:
    digest = hashlib.sha256()
    for path in sorted(set(paths), key=lambda item: item.as_posix()):
        try:
            label = path.relative_to(ROOT).as_posix()
        except ValueError:
            label = path.as_posix()
        digest.update(label.encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()[:20]


def normalize_fixture_mtimes(workspace: Path) -> None:
    """Keep identical cached fixture inputs older than their saved Cargo outputs."""
    for path in workspace.rglob("*"):
        if path.is_file():
            os.utime(path, (FIXTURE_MTIME_EPOCH, FIXTURE_MTIME_EPOCH))


def fixture_workspace_path(shared_target_dir: Path) -> Path:
    """Use a stable path outside the repository's vendored Cargo config."""
    parent = Path(os.environ.get("RUNNER_TEMP", str(shared_target_dir.parent))).resolve()
    if parent == ROOT or ROOT in parent.parents:
        parent = shared_target_dir.parent.resolve()
    path = parent / "rustui-kit09-fixture-workspace"
    if path == ROOT or ROOT in path.parents:
        raise FixtureError(f"fixture workspace must be outside the repository: {path}")
    return path


def compile_candidate_workspace(
    stage: Path,
    target_sources: dict[str, tuple[str, str]],
    manifest: dict[str, Any],
    fixture_root: Path,
) -> list[tuple[str, int, str]]:
    """Compile Base and Component from the verified archives plus applied stack."""
    workspace = fixture_root / "compile-workspace"
    sources_root = workspace / "sources"
    source_by_name = {source["name"]: source for source in manifest["sources"]}
    for source in manifest["sources"]:
        name, version = source["name"], source["version"]
        archive_path = fixture_root / "archives" / f"{name}-{version}.crate"
        destination = sources_root / f"{name}-{version}"
        prefix = f"{name}-{version}/"
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive.getmembers():
                if not member.isfile() or not member.name.startswith(prefix):
                    continue
                relative = Path(member.name[len(prefix):])
                if relative.is_absolute() or ".." in relative.parts:
                    raise FixtureError(f"unsafe path in crate archive: {member.name}")
                extracted = archive.extractfile(member)
                if extracted is None:
                    raise FixtureError(f"cannot read crate archive member: {member.name}")
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(extracted.read())

    for patch_target, (package, relative) in target_sources.items():
        source = source_by_name[package]
        source_path = sources_root / f"{package}-{source['version']}" / relative
        source_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(stage / patch_target, source_path)

    members = [
        f'"sources/{name}-{source_by_name[name]["version"]}"'
        for name in ("gpui-base", "gpui-component")
    ]
    proof_all = "--proof-a3-4a" in sys.argv[1:]
    proof_calendar = "--proof-calendar" in sys.argv[1:]
    proof_placeholder = "--proof-placeholder" in sys.argv[1:]
    proof_toolbar = "--proof-toolbar" in sys.argv[1:]
    proof_a3_4b = "--proof-a3-4b" in sys.argv[1:]
    proof_a3_4c = "--proof-a3-4c" in sys.argv[1:]
    proof_a3_4d = "--proof-a3-4d" in sys.argv[1:]
    proof_a3_4e = "--proof-a3-4e" in sys.argv[1:]
    proof_a3_4f = "--proof-a3-4f" in sys.argv[1:]
    proof_a3_4g = "--proof-a3-4g" in sys.argv[1:]
    proof_a3_4i = "--proof-a3-4i" in sys.argv[1:]
    proof_calendar = proof_calendar or proof_a3_4e
    proof_placeholder = proof_placeholder or proof_a3_4f
    proof_toolbar = proof_toolbar or proof_a3_4g
    if proof_a3_4i:
        members.append(f'"sources/gpui-pre-macos-{source_by_name["gpui-pre-macos"]["version"]}"')
    if "--proof-grid-a11y" in sys.argv[1:] or proof_all or proof_a3_4c:
        gpui_pre = sources_root / f'gpui-pre-{source_by_name["gpui-pre"]["version"]}'
        with (gpui_pre / "Cargo.toml").open("a", encoding="utf-8") as cargo_toml:
            cargo_toml.write("\n[dev-dependencies]\nutil_macros = { path = \"../../util_macros\" }\n")
        util_macros = workspace / "util_macros"
        (util_macros / "src").mkdir(parents=True, exist_ok=True)
        (util_macros / "Cargo.toml").write_text(
            "[package]\nname = \"util_macros\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\nproc-macro = true\n",
            encoding="utf-8",
        )
        (util_macros / "src/lib.rs").write_text(
            "use proc_macro::TokenStream;\n#[proc_macro_attribute]\npub fn perf(_: TokenStream, item: TokenStream) -> TokenStream { item }\n",
            encoding="utf-8",
        )
        (workspace / "assets/fonts/ibm-plex-sans").mkdir(parents=True, exist_ok=True)
        (workspace / "assets/fonts/lilex").mkdir(parents=True, exist_ok=True)
        (workspace / "assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf").write_bytes(b"")
        (workspace / "assets/fonts/lilex/Lilex-Regular.ttf").write_bytes(b"")
        members.append(f'"sources/gpui-pre-{source_by_name["gpui-pre"]["version"]}"')
        members.append('"util_macros"')
    if proof_all or proof_calendar or proof_placeholder or proof_toolbar or proof_a3_4b or proof_a3_4c or proof_a3_4d:
        proof_dir = workspace / "family-proofs"
        shutil.copytree(FAMILY_PROOF_CRATE / "tests", proof_dir / "tests")
        (proof_dir / "Cargo.toml").write_text(
            "[package]\nname = \"a2-4a-modal-proof\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n"
            "[dependencies]\n"
            f"gpui = {{ package = \"gpui-pre\", path = \"../sources/gpui-pre-{source_by_name['gpui-pre']['version']}\", default-features = false, features = [\"test-support\"] }}\n"
            f"gpui-base = {{ path = \"../sources/gpui-base-{source_by_name['gpui-base']['version']}\", default-features = false }}\n"
            f"gpui-component = {{ path = \"../sources/gpui-component-{source_by_name['gpui-component']['version']}\", default-features = false, features = [\"test-support\"] }}\n"
            "chrono = \"0.4\"\nserde_json = \"1\"\n",
            encoding="utf-8",
        )
        members.append('"family-proofs"')
    patch_entries = [
        f'"{name}" = {{ path = "sources/{name}-{source["version"]}" }}'
        for name, source in sorted(source_by_name.items())
    ]
    cargo_toml = (
        "[workspace]\nresolver = \"2\"\nmembers = ["
        + ", ".join(members + [f'"sources/gpui-pre-{source_by_name["gpui-pre"]["version"]}"'])
        + "]\n\n[patch.crates-io]\n"
        + "\n".join(patch_entries)
        + "\n"
    )
    (workspace / "Cargo.toml").write_text(cargo_toml, encoding="utf-8")
    if any(
        flag in sys.argv[1:]
        for flag in (
            "--proof-disabled-menu",
            "--proof-select-rtl",
            "--proof-tree-accessible-name",
            "--proof-tree-aria-level",
            "--proof-grid-a11y",
            "--proof-calendar",
            "--proof-placeholder",
            "--proof-toolbar",
            "--proof-a3-4a",
            "--proof-a3-4b",
            "--proof-a3-4c",
            "--proof-a3-4d",
            "--proof-a3-4e",
            "--proof-a3-4f",
            "--proof-a3-4g",
            "--proof-a3-4i",
        )
    ):
        # GPUI Component's crate archive omits several repository-level
        # include_str! fixtures that are only needed to compile its unit tests.
        test_assets = {
            "sources/assets/assets/icons/arrow-up.svg": "<svg/>",
            "sources/assets/assets/icons/search.svg": "<svg/>",
            "sources/base/src/dock/mod.rs": "",
            "examples/fixtures/test.md": "fixture",
            "themes/aurora.json": "{}",
        }
        for relative, content in test_assets.items():
            test_asset = workspace / relative
            test_asset.parent.mkdir(parents=True, exist_ok=True)
            test_asset.write_text(content, encoding="utf-8")
    shared_target_dir = Path(
        os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target/headstart"))
    ).resolve()
    target_dir = fixture_target_root(9, shared_target_dir) / FIXTURE_BUILD_CACHE / candidate_build_key()
    target_dir.mkdir(parents=True, exist_ok=True)
    print(f"fixture CARGO_TARGET_DIR={target_dir}")
    normalize_fixture_mtimes(workspace)
    command = ["rustup", "run", "1.98.0", "cargo", "check"]
    offline = "--compile-online" not in sys.argv[1:]
    if offline:
        command.append("--offline")
    command.extend(["-p", "gpui-base", "-p", "gpui-component"])
    if proof_a3_4i:
        command.extend(["-p", "gpui-pre", "-p", "gpui-pre-macos"])
    result = subprocess.run(
        command,
        cwd=workspace,
        env={**os.environ, "CARGO_TARGET_DIR": str(target_dir)},
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise FixtureError(
            f"compile gate failed ({' '.join(command[3:])}):\n"
            + result.stdout
            + result.stderr
        )
    print(f"compile gate: {' '.join(command[3:])}: passed")
    proofs: list[tuple[str, str, str, str | None]] = []
    if proof_all:
        proofs.extend(
            [
                ("backend_wrapped_rows_use_row_local_bidi_geometry_for_caret_and_hits", "gpui-pre", "A3-2 wrapped-row bidi", None),
                ("inert_scope_skips_tab_and_rejects_direct_focus", "gpui-pre", "A3-2 inert focus", None),
                ("test_write_a11y_info_disabled_state", "gpui-pre", "A3-2 disabled accessibility writer", None),
                ("modal_scope_restores_focus_and_restricts_traversal", "a2-4a-modal-proof", "A3-4a modal focus", "modal_scope"),
                ("tracked_dialog_can_be_queried_and_closed_by_id", "a2-4a-modal-proof", "A3-4a tracked dialog", "tracked_dialog"),
                ("sheet_close_accessibility_label_is_exposed_in_the_test_window_tree", "a2-4a-modal-proof", "A3-4a sheet close label", "sheet_accessibility"),
            ]
        )
    if proof_a3_4b:
        proofs.extend(
            [
                (
                    "disabled_selected_popup_menu_item_ignores_confirm",
                    "gpui-component",
                    "A3-4b disabled popup-menu confirmation",
                    None,
                ),
                (
                    "selection_recovery_skips_disabled_items_and_wraps",
                    "gpui-component",
                    "A3-4b popup-menu selection recovery",
                    None,
                ),
            ]
        )
    if proof_a3_4c:
        proofs.extend(
            [
                (
                    "data_grid_sort_and_readonly_metadata_reach_accessibility_nodes",
                    "gpui-pre",
                    "A3-4c GPUI ARIA sort/read-only metadata",
                    None,
                ),
                (
                    "tree_accessibility_label_and_levels_reach_the_tree_nodes",
                    "gpui-component",
                    "A3-4c Tree accessible name and levels",
                    None,
                ),
                (
                    "table_delegate_accessibility_nodes_keep_roles_indices_and_readonly",
                    "a2-4a-modal-proof",
                    "A3-4c DataTable delegate accessibility",
                    "grid_delegation",
                ),
            ]
        )
    if proof_a3_4d:
        proofs.extend(
            [
                (
                    "slider_accessibility_actions_are_advertised_and_clamp_to_range",
                    "a2-4a-modal-proof",
                    "A3-4d Slider SetValue/Increment/Decrement",
                    "slider_accessibility",
                ),
                (
                    "slider_set_value_uses_zero_origin_steps_and_preserves_range_start",
                    "a2-4a-modal-proof",
                    "A3-4d Slider SetValue preserves tested snapping",
                    "slider_accessibility",
                ),
                (
                    "slider_pointer_snapping_clamps_to_configured_maximum",
                    "a2-4a-modal-proof",
                    "A3-4d Slider endpoint and thumb geometry",
                    "slider_accessibility",
                ),
                (
                    "resize_handle_applies_the_first_pointer_position_immediately",
                    "a2-4a-modal-proof",
                    "A3-4d resize first pointer position",
                    "resize_panel",
                ),
                (
                    "resize_drag_finishes_when_left_button_is_no_longer_pressed",
                    "a2-4a-modal-proof",
                    "A3-4d resize release handling",
                    "resize_panel",
                ),
            ]
        )
    if proof_a3_4e:
        proofs.extend(
            [
                (
                    "calendar_month_names_include_french_and_arabic_translations",
                    "gpui-component",
                    "A3-4e French/Arabic calendar translations",
                    None,
                ),
                (
                    "calendar_keyboard_cursor_stays_uncommitted_until_activation",
                    "a2-4a-modal-proof",
                    "A3-4e calendar keyboard cursor",
                    "calendar_date_picker",
                ),
                (
                    "date_picker_arrow_cursor_activates_only_on_confirm",
                    "a2-4a-modal-proof",
                    "A3-4e DatePicker confirm behavior",
                    "calendar_date_picker",
                ),
                (
                    "rtl_multi_month_navigation_and_date_picker_accessibility_are_exposed",
                    "a2-4a-modal-proof",
                    "A3-4e RTL calendar and DatePicker accessibility",
                    "calendar_date_picker",
                ),
                (
                    "calendar_multi_month_count_stays_within_date_limits",
                    "a2-4a-modal-proof",
                    "A3-4e date limits and multi-month count",
                    "calendar_date_picker",
                ),
            ]
        )
    if proof_a3_4f:
        proofs.extend(
            [
                (
                    "placeholder_override_survives_palette_resolution",
                    "a2-4a-modal-proof",
                    "A3-4f placeholder color override",
                    "input_placeholder",
                ),
                (
                    "validation_description_reaches_the_editor_accessibility_node",
                    "a2-4a-modal-proof",
                    "A3-4f input validation description on editor node",
                    "input_placeholder",
                ),
            ]
        )
    if proof_a3_4g:
        proofs.append(
            (
                "rtl_toolbar_arrows_follow_visual_order",
                "a2-4a-modal-proof",
                "A3-4g RTL Toolbar visual-order navigation",
                "toolbar_rtl",
            )
        )
    if proof_a3_4i:
        proofs.extend(
            [
                (
                    "anchored_popup_placement_tests",
                    "gpui-pre-macos",
                    "A3-4i native popup placement",
                    None,
                ),
                (
                    "nested_grabbing_popups_dismiss_leaf_before_parent",
                    "gpui-pre-macos",
                    "A3-4i nested popup dismissal",
                    None,
                ),
                (
                    "parent_close_only_dismisses_managed_grabbing_popups",
                    "gpui-pre-macos",
                    "A3-4i parent closes only managed popups",
                    None,
                ),
            ]
        )
    outcomes: list[tuple[str, int, str]] = []
    for test_name, package, label, integration_test in proofs:
        proof_command = ["rustup", "run", "1.98.0", "cargo", "test"]
        if offline:
            proof_command.append("--offline")
        if integration_test is not None:
            proof_command.extend(["-p", package, "--test", integration_test, test_name])
        else:
            proof_command.extend(["-p", package, "--lib", test_name])
        if package == "gpui-pre":
            proof_command.extend(["--features", "test-support"])
        elif package == "gpui-component":
            proof_command.extend(["--features", "test-support"])
        elif package == "gpui-pre-macos":
            proof_command.extend(["--features", "test-support"])
        proof = subprocess.run(
            proof_command,
            cwd=workspace,
            env={**os.environ, "CARGO_TARGET_DIR": str(target_dir)},
            capture_output=True,
            text=True,
        )
        output = proof.stdout + proof.stderr
        outcomes.append((label, proof.returncode, output))
        if proof.returncode:
            print(f"proof: {label}: FAIL")
        else:
            print(f"proof: {label}: PASS")
    return outcomes


def run(write_manifest: bool = False) -> dict:
    manifest = load_manifest(DEFAULT_MANIFEST)
    proof_outcomes: list[tuple[str, int, str]] = []

    def run_compile_and_proofs(
        stage: Path,
        target_sources: dict[str, tuple[str, str]],
        fixture_manifest: dict[str, Any],
        fixture_root: Path,
    ) -> None:
        proof_outcomes.extend(
            compile_candidate_workspace(stage, target_sources, fixture_manifest, fixture_root)
        )

    shared_target_dir = Path(
        os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target/headstart"))
    ).resolve()
    fixture = fixture_workspace_path(shared_target_dir)
    if fixture.exists():
        shutil.rmtree(fixture)
    fixture.mkdir(parents=True)
    archive_count, target_count, unavailable_count = build_fixture(
        manifest, fixture, PATCH_DIR
    )
    result = apply_stack(
        manifest,
        PATCH_DIR,
        fixture,
        hashlib.sha256(DEFAULT_MANIFEST.read_bytes()).hexdigest(),
        patch_order=CORE_ORDER + FAMILY_ORDER,
        after_apply=(run_compile_and_proofs if "--compile" in sys.argv[1:] or "--compile-online" in sys.argv[1:] else None),
    )
    result["fixture"] = {
        "archives_verified": archive_count,
        "target_files_hashed": target_count,
        "known_unavailable_targets": unavailable_count,
    }
    data = (json.dumps(result, indent=2) + "\n").encode()
    if OUTPUT.exists() and not write_manifest and OUTPUT.read_bytes() != data:
        raise FixtureError(f"recorded stack manifest is stale: {OUTPUT}; use --write-manifest")
    OUTPUT.write_bytes(data)
    print(
        "A3-2 core patches={}; A3-4a patches={}; A3-4b patches={}; A3-4c patches={}; A3-4d patches={}; "
        "A3-4e patches={}; A3-4f patches={}; A3-4g patches={}; A3-4h patches={}; A3-4i patches={}; "
        "fixture archives verified={}; target files hashed={}; stack files={}; reverse=exact".format(
            len(CORE_ORDER),
            len(A3_4A_ORDER),
            len(A3_4B_ORDER),
            len(A3_4C_ORDER),
            len(A3_4D_ORDER),
            len(A3_4E_ORDER),
            len(A3_4F_ORDER),
            len(A3_4G_ORDER),
            len(A3_4H_ORDER),
            len(A3_4I_ORDER),
            archive_count,
            target_count,
            len(result["files"]),
        )
    )
    failed_proofs = [(label, output) for label, code, output in proof_outcomes if code]
    if failed_proofs:
        details = "\n\n".join(f"{label}:\n{output}" for label, output in failed_proofs)
        raise FixtureError(f"{len(failed_proofs)} cumulative family proof(s) failed after exact reverse:\n{details}")
    return result


if __name__ == "__main__":
    try:
        run("--write-manifest" in sys.argv[1:])
    except (FixtureError, OSError, subprocess.CalledProcessError) as error:
        print(f"Kit 0.9 family stack check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            if error.stdout:
                print(error.stdout, file=sys.stderr, end="")
            if error.stderr:
                print(error.stderr, file=sys.stderr, end="")
        raise SystemExit(1)
