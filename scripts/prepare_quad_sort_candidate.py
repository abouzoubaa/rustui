#!/usr/bin/env python3
"""Snapshot the approved quad-only experiment; never build or promote it."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import tomllib

PATCH = "2e1b35252aa120e5f7a7138f2c8a4b92aa9035f6edce7c4294333368d5909853"
PRE = "3addca59aab12ca0e00acb1e39d652863a8904146db2e28192275660aa5b8880"
POST = "08d04c593184213f433a046c01a77640a0ad83fa0c2d63d7588daee3ba485845"
PACKAGE = "gpui-pre-0.3.9"
INPUTS = ("Cargo.toml", "Cargo.lock", ".cargo", "crates", "apps", "examples",
          "locales", "themes", "scripts", "vendor")


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def inventory(root):
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        for name in dirs + files:
            p = Path(directory) / name
            if p.is_symlink():
                raise ValueError(f"symlink rejected: {p}")
        for name in files:
            p = Path(directory) / name
            result[p.relative_to(root).as_posix()] = digest(p)
    return dict(sorted(result.items()))


def tree_hash(files):
    return hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()


def write_json(path, data):
    with path.open("x") as stream:
        json.dump(data, stream, indent=2, sort_keys=True)
        stream.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate_id")
    args = parser.parse_args()
    if not re.fullmatch(r"[a-z][a-z0-9-]{0,63}", args.candidate_id):
        raise ValueError("candidate ID must be a safe lowercase identifier")
    root = Path(__file__).resolve().parents[1]
    parent = root / "target/headstart/candidates"
    parent.mkdir(parents=True, exist_ok=True)
    if not parent.resolve().is_relative_to(root):
        raise ValueError("candidate parent escapes repository")
    dest = parent / args.candidate_id
    dest.mkdir()  # Never overwrite, clean, resume, or reuse an experiment.
    evidence = dest / "evidence"
    evidence.mkdir()
    patch = root / "patches/proposed/gpui-pre-quad-stable-sort.patch"
    scene = root / "vendor" / PACKAGE / "src/scene.rs"
    assert digest(patch) == PATCH, "approved patch hash mismatch"
    assert digest(scene) == PRE, "pristine source mismatch"
    assert shutil.disk_usage(root).free > 8 * 1024**3, "less than 8 GiB free"
    active = {}
    print("Hashing workspace inputs and complete vendor closure", flush=True)
    for name in INPUTS:
        source = root / name
        if source.is_symlink():
            raise ValueError(f"symlink rejected: {source}")
        if source.is_file():
            active[name] = digest(source)
        else:
            active.update({f"{name}/{p}": h for p, h in inventory(source).items()})
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    locked = {(p["name"], p["version"]): p for p in lock["package"]}
    packages = []
    for package in sorted((root / "vendor").iterdir()):
        if not package.is_dir():
            continue
        metadata = tomllib.loads((package / "Cargo.toml").read_text())["package"]
        checksum = json.loads((package / ".cargo-checksum.json").read_text())
        files = {p[len(f"vendor/{package.name}/"):]: h for p, h in active.items()
                 if p.startswith(f"vendor/{package.name}/")}
        for name, expected in checksum["files"].items():
            if not (package / name).resolve().is_relative_to(package.resolve()):
                raise ValueError(f"checksum path escapes package: {name}")
            assert files.get(name) == expected, f"checksum mismatch: {package.name}/{name}"
        record = locked.get((metadata["name"], metadata["version"]))
        if record and record.get("checksum"):
            assert record["checksum"] == checksum["package"], package.name
        licenses = {p: h for p, h in files.items()
                    if any(x in Path(p).name.lower() for x in ("license", "copying", "copyright", "notice"))}
        packages.append({"directory": package.name, "name": metadata["name"],
                         "version": metadata["version"], "license": metadata.get("license", "unknown"),
                         "repository": metadata.get("repository", "unknown"),
                         "lock": record, "package_checksum": checksum["package"],
                         "tree_sha256": tree_hash(files), "license_files": licenses})
    shutil.copy2(patch, dest / "approved.patch")
    snapshots = {}
    for variant in ("baseline", "candidate"):
        print(f"Copying and verifying {variant}", flush=True)
        source = dest / variant / "source"
        source.mkdir(parents=True)
        (dest / variant / "target").mkdir()
        for name in INPUTS:
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
        application = None
        if variant == "candidate":
            assert digest(path_source / "src/scene.rs") == PRE
            command = ["patch", "--batch", "--fuzz=0", "-p3", "-d", str(path_source), "-i", str(dest / "approved.patch")]
            dry = subprocess.run(command + ["--dry-run"], capture_output=True, text=True)
            assert dry.returncode == 0 and not re.search(r"offset|fuzz|FAILED", dry.stdout + dry.stderr, re.I), dry.stdout + dry.stderr
            applied = subprocess.run(command, capture_output=True, text=True)
            assert applied.returncode == 0 and not re.search(r"offset|fuzz|FAILED", applied.stdout + applied.stderr, re.I), applied.stdout + applied.stderr
            assert digest(path_source / "src/scene.rs") == POST
            application = {"command": command, "dry_run": dry.stdout + dry.stderr,
                           "output": applied.stdout + applied.stderr, "exit_status": applied.returncode}
        result = inventory(path_source)
        differences = [p for p in set(pristine) | set(result) if pristine.get(p) != result.get(p)]
        assert differences == (["src/scene.rs"] if variant == "candidate" else []), differences
        manifest = source / "Cargo.toml"
        with manifest.open("a") as stream:
            stream.write('\n[patch.crates-io]\ngpui-pre = { path = "path-sources/gpui-pre-0.3.9" }\n')
        # The path override removes only gpui-pre's registry identity. All other
        # lock entries remain byte-for-byte equal; Cargo still verifies --locked.
        lock_path = source / "Cargo.lock"
        sections = lock_path.read_text().split("[[package]]")
        changed = 0
        for i, section in enumerate(sections):
            if re.search(r'^name = "gpui-pre"$', section, re.M) and re.search(r'^version = "0.3.9"$', section, re.M):
                sections[i] = re.sub(r'^(source|checksum) = .*\n', '', section, flags=re.M)
                changed += 1
        assert changed == 1
        lock_path.write_text("[[package]]".join(sections))
        snapshots[variant] = {"source": str(source), "target": str(dest / variant / "target"),
                              "files": inventory(source), "pristine_path_tree_sha256": tree_hash(pristine),
                              "path_tree_sha256": tree_hash(result), "application": application}
    print("Rechecking live inputs", flush=True)
    for name, expected in active.items():
        assert digest(root / name) == expected, f"live input changed while copying: {name}"
    baseline = snapshots["baseline"]["files"]
    candidate = snapshots["candidate"]["files"]
    assert [p for p in baseline if baseline[p] != candidate[p]] == [f"path-sources/{PACKAGE}/src/scene.rs"]
    record = {"schema_version": 1, "candidate_id": args.candidate_id,
              "created_unix": time.time(), "repository": str(root), "patch_sha256": PATCH,
              "preimage_sha256": PRE, "postimage_sha256": POST, "active_inputs": active,
              "packages": packages, "snapshots": snapshots,
              "copy_method": "shutil.copy2; independent files, no hardlinks or symlinks",
              "cargo": "not_run", "native": "not_run", "performance": "not_run",
              "toolchain_policy": "copied experimental cargo-headstart; caller flags preserved; metadata-crate-hash=no",
              "features": "workspace defaults; test-support closure to be resolved by authorized Cargo runner",
              "promotion": "not_authorized"}
    write_json(dest / "manifest.json", record)
    (dest / "manifest.json").chmod(0o444)
    write_json(evidence / "0001-preparation.json", {"manifest_sha256": digest(dest / "manifest.json"),
               "status": "prepared", "checksums_verified": len(packages), "cargo": "not_run"})
    print(json.dumps({"root": str(dest), "manifest_sha256": digest(dest / "manifest.json"),
                      "vendor_packages": len(packages), "input_files": len(active)}, indent=2))


if __name__ == "__main__":
    main()
