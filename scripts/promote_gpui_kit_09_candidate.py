#!/usr/bin/env python3
"""Promote the proven Kit 0.9 candidate stack into patches/approved/kit09."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from promote_gpui_kit_08_candidate import normalize_patch, target_paths
from prepare_gpui_kit_09_fixture import DEFAULT_MANIFEST, FixtureError, load_manifest, target_source_path


ROOT = Path(__file__).resolve().parents[1]
CANDIDATE = ROOT / "patches/candidate-kit-0.9"
CANDIDATE_STACK = CANDIDATE / "family-manifest.json"
OUTPUT = ROOT / "patches/approved/kit09"
# These dependency patches are already active on main and their sources do not
# change in the Kit 0.9 upgrade. They remain governed by their existing manifests.
PREEXISTING_PATCHES = {"block-0.1.6-stack-symbol-v1.patch", "cosmic-text-initial-bidi-span-v1.patch"}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def promote(fixture: Path, manifest_path: Path, output_dir: Path) -> tuple[int, int]:
    manifest = load_manifest(manifest_path)
    candidate_stack = json.loads(CANDIDATE_STACK.read_text(encoding="utf-8"))
    source_by_name = {source["name"]: source for source in manifest["sources"]}
    versions = {name: source["version"] for name, source in source_by_name.items()}
    package_names = sorted(source_by_name, key=len, reverse=True)
    order = [name for name in candidate_stack["order"] if name not in PREEXISTING_PATCHES]
    output_dir.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="rustui-kit09-promote-") as temporary:
        stage = Path(temporary)
        patch_records = []
        stack_targets: dict[str, tuple[str, str]] = {}
        normalized_patches: list[tuple[str, bytes]] = []
        for name in order:
            candidate = CANDIDATE / name
            if not candidate.is_file():
                raise FixtureError(f"candidate patch is missing: {candidate}")
            normalized = normalize_patch(candidate, package_names, versions)
            for target in target_paths(candidate):
                resolved = target_source_path(target, package_names)
                if resolved is None:
                    raise FixtureError(f"candidate target is not pinned: {target}")
                package, relative = resolved
                canonical = f"vendor/{package}-{versions[package]}/{relative}"
                stack_targets[canonical] = resolved
            normalized_patches.append((name, normalized))

        initial = {}
        for target, (package, relative) in sorted(stack_targets.items()):
            source = fixture / "crates" / f"{package}-{versions[package]}" / relative
            if not source.is_file():
                raise FixtureError(f"fixture file missing for {package}/{relative}")
            staged = stage / target
            staged.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, staged)
            initial[target] = source.read_bytes()

        for name, normalized in normalized_patches:
            target_list = sorted(
                line[6:] for line in normalized.decode("utf-8").splitlines() if line.startswith("+++ b/")
            )
            patch_file = output_dir / name
            patch_file.write_bytes(normalized)
            before = {target: (stage / target).read_bytes() for target in target_list}
            subprocess.run(["git", "apply", "--check", str(patch_file)], cwd=stage, check=True)
            subprocess.run(["git", "apply", str(patch_file)], cwd=stage, check=True)
            files = {
                target: {"pre_sha256": sha(before[target]), "post_sha256": sha((stage / target).read_bytes())}
                for target in target_list
            }
            patch_manifest = {
                "schema_version": 1,
                "upstream": manifest["upstream"],
                "candidate": name,
                "patch_sha256": sha(normalized),
                "files": files,
            }
            manifest_path_out = output_dir / f"{Path(name).stem}.json"
            manifest_path_out.write_text(json.dumps(patch_manifest, indent=2) + "\n", encoding="utf-8")
            patch_records.append({"patch": name, "patch_sha256": patch_manifest["patch_sha256"], "files": files})

        final = {target: (stage / target).read_bytes() for target in initial}
        for name, _ in reversed(normalized_patches):
            patch_file = output_dir / name
            subprocess.run(["git", "apply", "--reverse", "--check", str(patch_file)], cwd=stage, check=True)
            subprocess.run(["git", "apply", "--reverse", str(patch_file)], cwd=stage, check=True)
        if any((stage / target).read_bytes() != initial[target] for target in initial):
            raise FixtureError("promoted stack reverse did not restore fixture sources")

    stack = {
        "schema_version": 1,
        "upstream": manifest["upstream"],
        "source_manifest_sha256": sha(manifest_path.read_bytes()),
        "preexisting_patches": sorted(PREEXISTING_PATCHES),
        "order": order,
        "patches": patch_records,
        "files": {
            target: {"pre_sha256": sha(initial[target]), "post_sha256": sha(final[target])}
            for target in sorted(initial)
        },
        "reverse_verified": True,
    }
    (output_dir / "kit09-stack.json").write_text(json.dumps(stack, indent=2) + "\n", encoding="utf-8")
    return len(order), len(initial)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output-dir", type=Path, default=OUTPUT)
    args = parser.parse_args()
    try:
        patches, files = promote(args.fixture_dir, args.manifest, args.output_dir)
    except (FixtureError, OSError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"Kit 0.9 candidate promotion failed: {error}", file=sys.stderr)
        return 1
    print(f"promoted Kit 0.9 patches={patches}; target files={files}; exact reverse=PASS")
    print("retained existing approved block/cosmic-text patches unchanged")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
