#!/usr/bin/env python3
"""Apply and reverse the GPUI Kit 0.8 core plus the A2-4a candidate stack."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

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
