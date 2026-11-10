#!/usr/bin/env python3
"""Normalize candidate patches and validate them sequentially against Kit 0.8.1."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from check_kit08_kit_stack import CORE_ORDER, FAMILY_ORDER
from check_kit08_core_stack import patch_targets
from prepare_gpui_kit_08_fixture import DEFAULT_MANIFEST, FixtureError, load_manifest, target_source_path

ROOT = Path(__file__).resolve().parents[1]
CANDIDATE = ROOT / "patches/candidate-kit-0.8"


def target_paths(path: Path) -> list[str]:
    targets = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("+++ b/"):
            targets.add(line[6:])
        elif line.startswith("diff --git ") and " b/" in line:
            targets.add(line.split(" b/", 1)[1])
    return sorted(targets)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def normalize_patch(path: Path, package_names: list[str], versions: dict[str, str]) -> bytes:
    text = path.read_text(encoding="utf-8")
    chunks = re.split(r"(?=^diff --git |^--- a/)", text, flags=re.MULTILINE)
    by_logical: dict[tuple[str, str], list[str]] = {}
    order: list[str] = []
    for chunk in chunks:
        if not chunk.startswith(("diff --git ", "--- a/")):
            continue
        match = re.search(r"^\+\+\+ b/(.+)$", chunk, re.MULTILINE)
        if match:
            original = match.group(1)
        else:
            git_header = re.search(r"^diff --git a/.+ b/(.+)$", chunk, re.MULTILINE)
            if not git_header:
                raise FixtureError(f"patch target header missing in {path.name}")
            original = git_header.group(1)
        resolved = target_source_path(original, package_names)
        if resolved is None:
            raise FixtureError(f"candidate target is not pinned: {original}")
        package, relative = resolved
        target = f"vendor/{package}-{versions[package]}/{relative}"
        old_file = re.search(r"^--- a/(.+)$", chunk, re.MULTILINE)
        if old_file:
            chunk = chunk.replace(f"a/{old_file.group(1)}", f"a/{target}")
        old_git = re.search(r"^diff --git a/.+ b/.+$", chunk, re.MULTILINE)
        if old_git:
            chunk = re.sub(r"^diff --git a/.+ b/.+$", f"diff --git a/{target} b/{target}", chunk, count=1, flags=re.MULTILINE)
            if not re.search(r"^\+\+\+ b/", chunk, re.MULTILINE):
                hunk_start = re.search(r"^@@ ", chunk, re.MULTILINE)
                if hunk_start:
                    chunk = chunk[:hunk_start.start()] + f"--- a/{target}\n+++ b/{target}\n" + chunk[hunk_start.start():]
        chunk = chunk.replace(f"b/{original}", f"b/{target}")
        key = resolved
        previous = by_logical.setdefault(key, [])
        comparable = re.sub(r"^index .*\n", "", chunk, flags=re.MULTILINE)
        if any(re.sub(r"^index .*\n", "", old, flags=re.MULTILINE) == comparable for old in previous):
            continue
        previous.append(chunk)
        order.append(chunk)
    return "".join(order).encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    try:
        manifest = load_manifest(args.manifest)
        sources = {source["name"]: source for source in manifest["sources"]}
        versions = {name: source["version"] for name, source in sources.items()}
        package_names = sorted(sources, key=len, reverse=True)
        names = CORE_ORDER + FAMILY_ORDER
        args.output_dir.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="rustui-kit08-normalize-") as temp_name:
            stage = Path(temp_name)
            targets: dict[str, tuple[str, str]] = {}
            for name in names:
                for target in target_paths(CANDIDATE / name):
                    resolved = target_source_path(target, package_names)
                    if resolved is None:
                        raise FixtureError(f"candidate target is not pinned: {target}")
                    canonical = f"vendor/{resolved[0]}-{versions[resolved[0]]}/{resolved[1]}"
                    targets[canonical] = resolved
            for target, (package, relative) in targets.items():
                source = args.fixture_dir / "crates" / f"{package}-{versions[package]}" / relative
                if not source.is_file():
                    raise FixtureError(f"fixture source is missing: {package}/{relative}")
                destination = stage / target
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(source.read_bytes())
            initial = {target: (stage / target).read_bytes() for target in targets}
            stack_records = []
            for name in names:
                normalized = normalize_patch(CANDIDATE / name, package_names, versions)
                destination = args.output_dir / name
                destination.write_bytes(normalized)
                subprocess.run(["git", "apply", "--check", str(destination)], cwd=stage, check=True, capture_output=True, text=True)
                before = {target: (stage / target).read_bytes() for target in patch_targets(destination)}
                subprocess.run(["git", "apply", str(destination)], cwd=stage, check=True, capture_output=True, text=True)
                files = {}
                for target, old in before.items():
                    new = (stage / target).read_bytes()
                    files[target] = {"pre_sha256": sha(old), "post_sha256": sha(new)}
                patch_manifest = {
                    "schema_version": 1,
                    "upstream": manifest["upstream"],
                    "candidate": name,
                    "patch_sha256": sha(normalized),
                    "files": files,
                }
                (args.output_dir / f"{Path(name).stem}.json").write_text(json.dumps(patch_manifest, indent=2) + "\n", encoding="utf-8")
                stack_records.append({
                    "patch": name,
                    "patch_sha256": patch_manifest["patch_sha256"],
                    "files": files,
                })
            final = {target: (stage / target).read_bytes() for target in targets}
            for name in reversed(names):
                subprocess.run(["git", "apply", "--reverse", "--check", str(args.output_dir / name)], cwd=stage, check=True, capture_output=True, text=True)
                subprocess.run(["git", "apply", "--reverse", str(args.output_dir / name)], cwd=stage, check=True, capture_output=True, text=True)
            if any((stage / target).read_bytes() != initial[target] for target in targets):
                raise FixtureError("normalized stack reverse did not restore the pinned fixture")
            stack_manifest = {
                "schema_version": 1,
                "upstream": manifest["upstream"],
                "order": names,
                "patches": stack_records,
                "files": {
                    target: {"pre_sha256": sha(initial[target]), "post_sha256": sha(final[target])}
                    for target in sorted(targets)
                },
                "reverse_verified": True,
            }
            (args.output_dir / "kit08-stack.json").write_text(json.dumps(stack_manifest, indent=2) + "\n", encoding="utf-8")
    except (FixtureError, OSError, subprocess.CalledProcessError) as error:
        print(f"candidate normalization failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError):
            print(error.stderr or error.stdout, file=sys.stderr)
        return 1
    print(f"normalized candidate patches={len(names)}; manifests={len(names)}; exact reverse=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
