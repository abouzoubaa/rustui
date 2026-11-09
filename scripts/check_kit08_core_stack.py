#!/usr/bin/env python3
"""Check and reverse the inert GPUI Kit 0.8 core patch candidate stack."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

from prepare_gpui_kit_08_fixture import (
    DEFAULT_MANIFEST,
    FixtureError,
    build_fixture,
    load_manifest,
    target_source_path,
)


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_PATCH_DIR = ROOT / "patches/candidate-kit-0.8"
DEFAULT_OUTPUT = DEFAULT_PATCH_DIR / "core-manifest.json"
CANDIDATE_ORDER = (
    "block-0.1.6-stack-symbol-v1.patch",
    "cosmic-text-initial-bidi-span-v1.patch",
    "g10-parent-node-id-v1.patch",
    "g10-labelled-by-tree-v1-kit08.patch",
    "gpui-bidi-wrapped-rows-v1-kit08.patch",
    "gpui-bidi-input-hit-v1-kit08.patch",
    "gpui-macos-focused-element-forwarder-v1-kit08.patch",
    "gpui-pre-test-a11y-activation-v1.patch",
    "gpui-pre-web-accessible-name-v1.patch",
    "gpui-primary-mousedown-click-v2.patch",
    "r15-test-tree-disabled-v1-kit08.patch",
    "t17-inert-focus-scope-v1-kit08.patch",
    "gpui-tree-aria-disabled-v1-kit08.patch",
)
LATER_FAMILY_CANDIDATES = {
    "t13-modal-focus-recovery-v1-kit08.patch",
    "gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch",
    "gpui-component-sheet-close-accessibility-v2.patch",
    "r15-button-disabled-focus-v1-kit08.patch",
    "r15-sidebar-disabled-v1.patch",
    "r15-breadcrumb-disabled-v1.patch",
    "gpui-component-popup-menu-disabled-confirm-v1-kit08.patch",
    "r15b-menu-item-disabled-v1.patch",
    "r15b-popup-menu-recovery-v1.patch",
    "r15b-dropdown-menu-refresh-v1.patch",
    "gpui-component-select-popup-rtl-v1-kit08.patch",
    "gpui-component-select-rtl-alignment-v1.patch",
    "gpui-component-select-rtl-alignment-v2-kit08.patch",
    "gpui-component-select-rtl-alignment-v3-kit08.patch",
    "gpui-component-select-rtl-auto-width-v4-kit08.patch",
    "gpui-component-select-rtl-auto-width-v6-kit08.patch",
    "gpui-selectable-list-semantics-v2.patch",
    "gpui-component-tree-accessible-name-v1-kit08.patch",
    "gpui-tree-aria-disabled-row-v1-kit08.patch",
    "gpui-tree-aria-level-v1-kit08.patch",
    "gpui-tree-disabled-keyboard-v2.patch",
    "r15-tree-disabled-focus-v1.patch",
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def patch_targets(patch_path: Path) -> list[str]:
    return sorted(
        {
            line[6:]
            for line in patch_path.read_text(encoding="utf-8").splitlines()
            if line.startswith("+++ b/")
        }
    )


def apply_stack(
    manifest: dict[str, Any],
    patch_dir: Path,
    fixture_root: Path,
    source_manifest_sha256: str,
    patch_order: tuple[str, ...] | None = None,
    after_apply: Callable[[Path, dict[str, tuple[str, str]], dict[str, Any], Path], None] | None = None,
) -> dict[str, Any]:
    sources = {source["name"]: source for source in manifest["sources"]}
    package_names = sorted(sources, key=len, reverse=True)
    patch_paths = {path.name: path for path in patch_dir.glob("*.patch")}
    if patch_order is not None:
        unexpected = sorted(set(patch_paths) - set(patch_order))
        if unexpected:
            raise FixtureError("candidate patch order is undefined for: " + ", ".join(unexpected))
        patches = [patch_paths[name] for name in patch_order if name in patch_paths]
    elif patch_dir.resolve() == DEFAULT_PATCH_DIR.resolve():
        unexpected = sorted(
            set(patch_paths) - set(CANDIDATE_ORDER) - LATER_FAMILY_CANDIDATES
        )
        if unexpected:
            raise FixtureError("candidate patch order is undefined for: " + ", ".join(unexpected))
        patches = [patch_paths[name] for name in CANDIDATE_ORDER if name in patch_paths]
    else:
        patches = [patch_paths[name] for name in sorted(patch_paths)]
    if not patches:
        raise FixtureError(f"candidate patch directory is empty: {patch_dir}")

    target_sources: dict[str, tuple[str, str]] = {}
    for patch_path in patches:
        for target in patch_targets(patch_path):
            resolved = target_source_path(target, package_names)
            if resolved is None:
                raise FixtureError(f"candidate patch target is not in the pinned source set: {target}")
            target_sources[target] = resolved

    with tempfile.TemporaryDirectory(prefix="rustui-kit08-core-stack-") as stage_name:
        stage = Path(stage_name)
        initial: dict[str, bytes] = {}
        for target, (package, relative_path) in sorted(target_sources.items()):
            source = sources[package]
            fixture_path = (
                fixture_root
                / "sources"
                / f"{package}-{source['version']}"
                / relative_path
            )
            if not fixture_path.is_file():
                raise FixtureError(f"fixture source is missing: {package}/{relative_path}")
            data = fixture_path.read_bytes()
            target_path = stage / target
            target_path.parent.mkdir(parents=True, exist_ok=True)
            target_path.write_bytes(data)
            initial[target] = data

        patch_records: list[dict[str, Any]] = []
        for patch_path in patches:
            before = {target: (stage / target).read_bytes() for target in patch_targets(patch_path)}
            subprocess.run(
                ["git", "apply", "--check", str(patch_path)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
            subprocess.run(
                ["git", "apply", str(patch_path)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
            after = {target: (stage / target).read_bytes() for target in before}
            patch_records.append(
                {
                    "patch": patch_path.name,
                    "patch_sha256": sha256(patch_path.read_bytes()),
                    "files": {
                        target: {
                            "pre_sha256": sha256(before[target]),
                            "post_sha256": sha256(after[target]),
                        }
                        for target in sorted(before)
                    },
                }
            )

        if after_apply is not None:
            after_apply(stage, target_sources, manifest, fixture_root)

        final = {target: (stage / target).read_bytes() for target in initial}
        for patch_path in reversed(patches):
            subprocess.run(
                ["git", "apply", "--reverse", "--check", str(patch_path)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
            subprocess.run(
                ["git", "apply", "--reverse", str(patch_path)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
        restored = {target: (stage / target).read_bytes() for target in initial}
        mismatches = [target for target in initial if restored[target] != initial[target]]
        if mismatches:
            raise FixtureError("reverse stack did not restore the fixture: " + ", ".join(mismatches))

    return {
        "schema_version": 1,
        "upstream": manifest["upstream"],
        "source_manifest_sha256": source_manifest_sha256,
        "order": [patch.name for patch in patches],
        "patches": patch_records,
        "files": {
            target: {
                "pre_sha256": sha256(initial[target]),
                "post_sha256": sha256(final[target]),
            }
            for target in sorted(initial)
        },
        "reverse_verified": True,
    }


def run(
    manifest_path: Path = DEFAULT_MANIFEST,
    patch_dir: Path = DEFAULT_PATCH_DIR,
    output_path: Path = DEFAULT_OUTPUT,
    write_manifest: bool = False,
) -> dict[str, Any]:
    manifest = load_manifest(manifest_path)
    with tempfile.TemporaryDirectory(prefix="rustui-kit08-core-fixture-") as temp_name:
        fixture_root = Path(temp_name)
        core_inventory = fixture_root / "core-candidate-inventory"
        core_inventory.mkdir()
        inventory_order = (
            CANDIDATE_ORDER
            if patch_dir.resolve() == DEFAULT_PATCH_DIR.resolve()
            else tuple(path.name for path in sorted(patch_dir.glob("*.patch")))
        )
        for name in inventory_order:
            shutil.copyfile(patch_dir / name, core_inventory / name)
        archive_count, target_count, unavailable_count = build_fixture(
            manifest, fixture_root, core_inventory
        )
        result = apply_stack(
            manifest,
            patch_dir,
            fixture_root,
            sha256(manifest_path.read_bytes()),
        )
    result["fixture"] = {
        "archives_verified": archive_count,
        "target_files_hashed": target_count,
        "known_unavailable_targets": unavailable_count,
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output = (json.dumps(result, indent=2) + "\n").encode("utf-8")
    if output_path.exists() and not write_manifest:
        if output_path.read_bytes() != output:
            raise FixtureError(
                f"recorded stack manifest is stale: {output_path}; use --write-manifest to refresh"
            )
    else:
        output_path.write_bytes(output)
    print(
        "candidate patches={}; fixture archives verified={}; target files hashed={}; "
        "stack files={}; reverse=exact".format(
            len(result["order"]),
            archive_count,
            target_count,
            len(result["files"]),
        )
    )
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--patch-dir", type=Path, default=DEFAULT_PATCH_DIR)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument(
        "--write-manifest",
        action="store_true",
        help="write or refresh the recorded candidate source hashes",
    )
    args = parser.parse_args(argv)
    try:
        run(args.manifest, args.patch_dir, args.output, args.write_manifest)
    except (FixtureError, OSError, subprocess.CalledProcessError) as error:
        print(f"Kit 0.8 core stack check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            if error.stdout:
                print(error.stdout, file=sys.stderr, end="")
            if error.stderr:
                print(error.stderr, file=sys.stderr, end="")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
