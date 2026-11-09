#!/usr/bin/env python3
"""Apply and reverse the GPUI Kit 0.8 core plus the A2-4a candidate stack."""

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

from check_kit08_core_stack import apply_stack
from prepare_gpui_kit_08_fixture import (
    DEFAULT_MANIFEST,
    FixtureError,
    build_fixture,
    load_manifest,
)


ROOT = Path(__file__).resolve().parent.parent
PATCH_DIR = ROOT / "patches/candidate-kit-0.8"
OUTPUT = PATCH_DIR / "kit-manifest.json"
CORE_ORDER = (
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
)
FAMILY_ORDER = (
    "t13-modal-focus-recovery-v1-kit08.patch",
    "gpui-component-tracked-dialog-focus-rebase-v2-kit08.patch",
)


def compile_candidate_workspace(
    stage: Path,
    target_sources: dict[str, tuple[str, str]],
    manifest: dict[str, Any],
    fixture_root: Path,
) -> None:
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
    patch_entries = [
        f'"{name}" = {{ path = "sources/{name}-{source["version"]}" }}'
        for name, source in sorted(source_by_name.items())
    ]
    cargo_toml = (
        "[workspace]\nresolver = \"2\"\nmembers = ["
        + ", ".join(members)
        + "]\n\n[patch.crates-io]\n"
        + "\n".join(patch_entries)
        + "\n"
    )
    (workspace / "Cargo.toml").write_text(cargo_toml, encoding="utf-8")
    target_dir = fixture_root / "compile-target"
    command = ["rustup", "run", "1.97.1", "cargo", "check"]
    offline = "--compile-online" not in sys.argv[1:]
    if offline:
        command.append("--offline")
    command.extend(["-p", "gpui-base", "-p", "gpui-component"])
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


def run(write_manifest: bool = False) -> dict:
    manifest = load_manifest(DEFAULT_MANIFEST)
    with tempfile.TemporaryDirectory(prefix="rustui-kit08-kit-stack-") as temp_name:
        fixture = Path(temp_name)
        archive_count, target_count, unavailable_count = build_fixture(
            manifest, fixture, PATCH_DIR
        )
        result = apply_stack(
            manifest,
            PATCH_DIR,
            fixture,
            hashlib.sha256(DEFAULT_MANIFEST.read_bytes()).hexdigest(),
            patch_order=CORE_ORDER + FAMILY_ORDER,
            after_apply=(
                compile_candidate_workspace
                if "--compile" in sys.argv[1:] or "--compile-online" in sys.argv[1:]
                else None
            ),
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
        "core patches={}; A2-4a patches={}; fixture archives verified={}; "
        "target files hashed={}; stack files={}; reverse=exact".format(
            len(CORE_ORDER), len(FAMILY_ORDER), archive_count, target_count,
            len(result["files"]),
        )
    )
    return result


if __name__ == "__main__":
    try:
        run("--write-manifest" in sys.argv[1:])
    except (FixtureError, OSError, subprocess.CalledProcessError) as error:
        print(f"Kit 0.8 candidate stack check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            if error.stdout:
                print(error.stdout, file=sys.stderr, end="")
            if error.stderr:
                print(error.stderr, file=sys.stderr, end="")
        raise SystemExit(1)
