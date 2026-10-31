#!/usr/bin/env python3
"""Prepare the approved Dialog experiment without building or promotion."""

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib


ID = "dialog-focus-rebase-20261018-a"
PACKAGE = "gpui-component-0.7.3"
PATCH = "07a507ac52c21bb4f3205a4358b6b27458941e5274e91be4d31bfc482bbb3462"
PRE = {
    "src/root.rs": "c438aabc60f18252420a40e773e0aa4447fe89b00e27033c0ad7e5980b9c2749",
    "src/window_ext.rs": "697b8a6867d1f12085ac6c976568745f6ba4ca21f5a9727c7a74e30523ed77eb",
    "src/lib.rs": "50d77481bd99c62fa711689a2608add5bc355f700bdb72ebced910af8e62fda7",
    "src/dialog/dialog.rs": "0189f3a07238d2ad275c891c39efaaa0773817a7f8eb45b061eda12b219d86a1",
}


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def inventory(root, excluded=()):
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        if Path(directory) == root:
            dirs[:] = [name for name in dirs if name not in excluded]
        for name in dirs + files:
            path = Path(directory) / name
            if path.is_symlink():
                raise ValueError(f"symlink rejected: {path}")
        for name in files:
            path = Path(directory) / name
            if Path(directory) == root and name in excluded:
                continue
            result[path.relative_to(root).as_posix()] = digest(path)
    return dict(sorted(result.items()))


def tree_hash(files):
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


def write_json(path, data):
    with path.open("x") as stream:
        json.dump(data, stream, indent=2, sort_keys=True)
        stream.write("\n")


def copy_approved_patch(source, destination, expected_hash):
    if digest(source) != expected_hash:
        raise ValueError("approved patch changed before copy")
    shutil.copy2(source, destination)
    if digest(destination) != expected_hash:
        raise ValueError("approved patch changed during copy")


def main():
    root = Path(__file__).resolve().parents[1]
    parent = root / "target/headstart/candidates"
    parent.mkdir(parents=True, exist_ok=True)
    if not parent.resolve().is_relative_to(root):
        raise ValueError("candidate parent escapes repository")
    dest = parent / ID
    patch = root / "patches/proposed/gpui-component-tracked-dialog-focus-rebase.patch"
    assert digest(patch) == PATCH, "approved patch hash mismatch"
    for name, expected in PRE.items():
        assert digest(root / "vendor" / PACKAGE / name) == expected, name
    assert shutil.disk_usage(root).free > 8 * 1024**3, "less than 8 GiB free"
    inputs = sorted(path.name for path in root.iterdir() if path.name != "target")
    print("Hashing current repository inputs, excluding target", flush=True)
    active = inventory(root, excluded={"target"})

    lock = tomllib.loads((root / "Cargo.lock").read_text())
    locked = {(item["name"], item["version"]): item for item in lock["package"]}
    packages = []
    for package in sorted((root / "vendor").iterdir()):
        if not package.is_dir():
            continue
        metadata = tomllib.loads((package / "Cargo.toml").read_text())["package"]
        checksum = json.loads((package / ".cargo-checksum.json").read_text())
        files = {name[len(f"vendor/{package.name}/"):]: value
                 for name, value in active.items() if name.startswith(f"vendor/{package.name}/")}
        for name, expected in checksum["files"].items():
            if not (package / name).resolve().is_relative_to(package.resolve()):
                raise ValueError(f"checksum path escapes package: {name}")
            assert files.get(name) == expected, f"checksum mismatch: {package.name}/{name}"
        record = locked.get((metadata["name"], metadata["version"]))
        if record and record.get("checksum"):
            assert record["checksum"] == checksum["package"], package.name
        licenses = {name: value for name, value in files.items()
                    if any(term in Path(name).name.lower()
                           for term in ("license", "copying", "copyright", "notice"))}
        packages.append({"directory": package.name, "name": metadata["name"],
                         "version": metadata["version"], "license": metadata.get("license", "unknown"),
                         "repository": metadata.get("repository", "unknown"),
                         "lock": record, "package_checksum": checksum["package"],
                         "tree_sha256": tree_hash(files), "license_files": licenses})

    dest.mkdir()  # Never overwrite or reuse an experiment.
    evidence = dest / "evidence"
    evidence.mkdir()
    copy_approved_patch(patch, dest / "approved.patch", PATCH)
    snapshots = {}
    for variant in ("baseline", "candidate"):
        print(f"Copying and verifying {variant}", flush=True)
        source = dest / variant / "source"
        source.mkdir(parents=True)
        (dest / variant / "target").mkdir()
        for name in inputs:
            original, copied = root / name, source / name
            if original.is_dir():
                shutil.copytree(original, copied, copy_function=shutil.copy2)
            else:
                shutil.copy2(original, copied)
        assert inventory(source) == active, f"{variant} snapshot mismatch"
        path_source = source / "path-sources" / PACKAGE
        path_source.parent.mkdir()
        shutil.copytree(source / "vendor" / PACKAGE, path_source, copy_function=shutil.copy2)
        pristine = inventory(path_source)
        assert {name: pristine[name] for name in PRE} == PRE
        application = None
        if variant == "candidate":
            command = ["patch", "--batch", "--fuzz=0", "-p3", "-d", str(path_source),
                       "-i", str(dest / "approved.patch")]
            dry = subprocess.run(command + ["--dry-run"], capture_output=True, text=True)
            assert dry.returncode == 0 and not re.search(r"offset|fuzz|FAILED", dry.stdout + dry.stderr, re.I), dry.stdout + dry.stderr
            applied = subprocess.run(command, capture_output=True, text=True)
            assert applied.returncode == 0 and not re.search(r"offset|fuzz|FAILED", applied.stdout + applied.stderr, re.I), applied.stdout + applied.stderr
            application = {"command": command, "dry_run": dry.stdout + dry.stderr,
                           "output": applied.stdout + applied.stderr, "exit_status": applied.returncode}
        result = inventory(path_source)
        differences = sorted(name for name in set(pristine) | set(result)
                             if pristine.get(name) != result.get(name))
        assert differences == (sorted(PRE) if variant == "candidate" else []), differences
        with (source / "Cargo.toml").open("a") as stream:
            stream.write(f'\n[patch.crates-io]\ngpui-component = {{ path = "path-sources/{PACKAGE}" }}\n')
        lock_path = source / "Cargo.lock"
        sections = lock_path.read_text().split("[[package]]")
        changed = 0
        for index, section in enumerate(sections):
            if re.search(r'^name = "gpui-component"$', section, re.M) and re.search(r'^version = "0.7.3"$', section, re.M):
                sections[index] = re.sub(r'^(source|checksum) = .*\n', '', section, flags=re.M)
                changed += 1
        assert changed == 1
        lock_path.write_text("[[package]]".join(sections))
        snapshots[variant] = {"source": str(source), "target": str(dest / variant / "target"),
                              "files": inventory(source), "pristine_path_tree_sha256": tree_hash(pristine),
                              "path_tree_sha256": tree_hash(result), "patch_differences": differences,
                              "postimages": {name: result[name] for name in PRE},
                              "application": application}

    print("Rechecking live inputs", flush=True)
    for name, expected in active.items():
        assert digest(root / name) == expected, f"live input changed while copying: {name}"
    baseline, candidate = (snapshots[name]["files"] for name in ("baseline", "candidate"))
    assert sorted(name for name in baseline if baseline[name] != candidate[name]) == [
        f"path-sources/{PACKAGE}/{name}" for name in sorted(PRE)]
    record = {"schema_version": 1, "candidate_id": ID, "created_unix": time.time(),
              "repository": str(root), "patch_sha256": PATCH, "preimages": PRE,
              "active_inputs": active, "packages": packages, "snapshots": snapshots,
              "copy_method": "shutil.copy2; independent files, no hardlinks or symlinks",
              "cargo": "not_run", "native": "not_run", "gui": "not_run",
              "toolchain_policy": "experimental scripts/cargo-headstart; caller flags preserved; metadata-crate-hash=no",
              "toolchain_file_sha256": active["rust-toolchain.toml"],
              "launcher_sha256": active["scripts/cargo-headstart"],
              "promotion": "not_authorized"}
    write_json(dest / "manifest.json", record)
    (dest / "manifest.json").chmod(0o444)
    write_json(evidence / "0001-preparation.json", {"status": "prepared",
               "manifest_sha256": digest(dest / "manifest.json"),
               "checksums_verified": len(packages), "cargo": "not_run", "gui": "not_run"})
    print(json.dumps({"root": str(dest), "manifest_sha256": digest(dest / "manifest.json"),
                      "vendor_packages": len(packages), "input_files": len(active)}, indent=2))


if __name__ == "__main__":
    main()
