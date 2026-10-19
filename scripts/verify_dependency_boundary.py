#!/usr/bin/env python3
"""Check RustUI workspace dependency metadata for forbidden boundaries."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
FORBIDDEN_PACKAGE = "allui"


def cargo_metadata(no_deps: bool) -> dict:
    command = [str(ROOT / "scripts/cargo-headstart"), "metadata", "--format-version", "1"]
    if no_deps:
        command.append("--no-deps")
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or f"cargo metadata exited {result.returncode}")
    return json.loads(result.stdout)


def check() -> list[str]:
    workspace = cargo_metadata(no_deps=True)
    resolved = cargo_metadata(no_deps=False)
    root = Path(workspace["workspace_root"]).resolve()
    member_ids = set(workspace["workspace_members"])
    declared = [p for p in workspace["packages"] if p["id"] in member_ids]
    packages = {p["id"]: p for p in resolved["packages"]}
    errors: list[str] = []

    for package in declared:
        for dep in package["dependencies"]:
            if dep["name"].casefold() == FORBIDDEN_PACKAGE:
                errors.append(f"{package['name']}: declared dependency on Allui ({dep['name']})")
            if dep.get("path"):
                path = Path(dep["path"]).resolve()
                if not path.is_relative_to(root):
                    errors.append(f"{package['name']}: path dependency outside workspace: {dep['name']} -> {path}")

    pending = list(resolved["workspace_members"])
    visited: set[str] = set()
    while pending:
        package_id = pending.pop()
        if package_id in visited:
            continue
        visited.add(package_id)
        package = packages[package_id]
        for dep in package["dependencies"]:
            dep_name = dep.get("name")
            if dep_name and dep_name.casefold() == FORBIDDEN_PACKAGE:
                errors.append(f"{package['name']}: declared dependency on Allui ({dep_name})")
            if dep.get("path"):
                path = Path(dep["path"]).resolve()
                if not path.is_relative_to(root):
                    errors.append(f"{package['name']}: declared path dependency outside workspace: {dep['name']} -> {path}")
        for dep in package["dependencies"]:
            target = packages.get(dep.get("pkg"))
            if target is None:
                continue
            if target["name"].casefold() == FORBIDDEN_PACKAGE:
                errors.append(f"{package['name']}: resolved dependency on Allui ({target['name']})")
            if target.get("source") is None:
                path = Path(target["manifest_path"]).resolve().parent
                if not path.is_relative_to(root):
                    errors.append(f"{package['name']}: resolved local dependency outside workspace: {target['name']} -> {path}")
            pending.append(target["id"])
    return errors


def main() -> int:
    try:
        errors = check()
    except (OSError, RuntimeError, ValueError, KeyError, TypeError) as exc:
        print(f"Dependency boundary check could not complete: {exc}", file=sys.stderr)
        return 2
    if errors:
        print("RustUI dependency boundary violations:", file=sys.stderr)
        for error in sorted(set(errors)):
            print(f"- {error}", file=sys.stderr)
        return 1
    print("RustUI dependency boundary check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
