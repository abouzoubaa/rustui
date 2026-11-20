#!/usr/bin/env python3
"""Create/check/reverse owned Fast copies using pinned source and patch images."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

from apply_f0_fast_candidate_patch import run as apply_patch

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / "patches/approved/fast/owned-fast-sources.json"
WIRING = ROOT / "patches/approved/fast/owned-fast-path-wiring-v1.json"
DAMAGE_STACK = ROOT / "patches/candidate-fast-damage/damage-stack.json"


def source_images(copy, stack_path=None):
    """The pinned approved snapshot, followed by the selected damage patches."""
    expected = dict(copy["files"])
    stack_path = stack_path or DAMAGE_STACK
    if stack_path.exists():
        from check_fast_patch_stack import load_manifests
        entries, _ = load_manifests(stack_path)
        prefix = copy["vendor"] + "/"
        for path, _, files in entries:
            if path.parent == WIRING.parent:
                continue
            for name, hashes in files.items():
                if name.startswith(prefix):
                    relative = name[len(prefix):]
                    if expected.get(relative) != hashes["pre_sha256"]:
                        raise ValueError(f"damage source chain differs: {name}")
                    expected[relative] = hashes["post_sha256"]
    return expected


def manifest_wiring():
    names = json.loads((WIRING.parent / "fast-stack.json").read_text())["patches"]
    names = names[names.index(WIRING.name):]
    paths = [WIRING.parent / name for name in names]
    for path in paths:
        data = json.loads(path.read_text())
        if any(not name.endswith("/Cargo.toml") for name in data["files"]):
            raise ValueError(f"ownership tail must contain manifest-only patches: {path.name}")
        if hashlib.sha256((path.parent / data["patch"]).read_bytes()).hexdigest() != data["patch_sha256"]:
            raise ValueError(f"ownership patch hash mismatch: {path.name}")
    return paths


@contextmanager
def wired_manifests(root, stack_path=None):
    """Compose the pinned wiring diff with candidate vendor manifest changes.

    Hashes for generated owned manifests follow from exact candidate inputs and
    the unchanged approved wiring patch. The regular applier still proves both
    images and exact reversal before any real file is changed.
    """
    paths = manifest_wiring()
    copies = json.loads(SOURCES.read_text())["copies"]
    if all(source_images(copy, stack_path)["Cargo.toml"] == copy["files"]["Cargo.toml"]
           for copy in copies):
        yield paths
        return
    with tempfile.TemporaryDirectory(prefix="rustui-fast-wiring-") as temporary:
        stage = Path(temporary)
        owned = set()
        for copy in copies:
            name = copy["owned"] + "/Cargo.toml"
            owned.add(name)
            data = (root / copy["vendor"] / "Cargo.toml").read_bytes()
            if hashlib.sha256(data).hexdigest() != source_images(copy, stack_path)["Cargo.toml"]:
                raise ValueError(f"candidate vendor manifest differs: {copy['vendor']}")
            target = stage / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        effective = []
        for path in paths:
            data = json.loads(path.read_text())
            patch = path.parent / data["patch"]
            selected = owned.intersection(data["files"])
            before = {name: (stage / name).read_bytes() for name in selected}
            if selected:
                include = [f"--include={name}" for name in sorted(selected)]
                command = ["git", "apply", "--whitespace=nowarn", *include]
                subprocess.run(command + [str(patch)], cwd=stage, check=True, capture_output=True)
                after = {name: (stage / name).read_bytes() for name in selected}
                subprocess.run(command + ["--reverse", str(patch)], cwd=stage, check=True, capture_output=True)
                if any((stage / name).read_bytes() != value for name, value in before.items()):
                    raise ValueError(f"candidate wiring did not reverse exactly: {path.name}")
                for name, value in after.items():
                    (stage / name).write_bytes(value)
                    data["files"][name] = {
                        "pre_sha256": hashlib.sha256(before[name]).hexdigest(),
                        "post_sha256": hashlib.sha256(value).hexdigest(),
                    }
            data["patch"] = str(patch.resolve())
            target = stage / path.name
            target.write_text(json.dumps(data))
            effective.append(target)
        yield effective


def check_manifest_images(root, stack_path=None):
    final = {}
    with wired_manifests(root, stack_path) as paths:
        for path in paths:
            final.update({name: hashes["post_sha256"]
                          for name, hashes in json.loads(path.read_text())["files"].items()})
    for name, expected in final.items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != expected:
            raise ValueError(f"owned manifest postimage differs: {name}")


def hashes(directory: Path):
    result = {}
    for path in sorted(directory.rglob("*")):
        if path.is_symlink():
            raise ValueError(f"unexpected owned/source symlink: {path}")
        if path.is_file():
            result[path.relative_to(directory).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def check_copies(root: Path, wired: bool, stack_path=None):
    copies = json.loads(SOURCES.read_text())["copies"]
    wiring = {}
    with wired_manifests(root, stack_path) as paths:
        for path in paths:
            wiring.update(json.loads(path.read_text())["files"])
    for copy in copies:
        expected = source_images(copy, stack_path)
        if hashes(root / copy["vendor"]) != expected:
            raise ValueError(f"pinned Fast vendor snapshot differs: {copy['vendor']}")
        owned_expected = dict(expected)
        manifest = copy["owned"] + "/Cargo.toml"
        if wired and manifest in wiring:
            owned_expected["Cargo.toml"] = wiring[manifest]["post_sha256"]
        if hashes(root / copy["owned"]) != owned_expected:
            raise ValueError(f"owned Fast snapshot differs: {copy['owned']}")
    print(f"Owned Fast copies={len(copies)}; complete source/license snapshots=PASS; wired={wired}")


def operate(action: str, root: Path = ROOT, stack_path=None):
    copies = json.loads(SOURCES.read_text())["copies"]
    if action == "--apply":
        for copy in copies:
            if hashes(root / copy["vendor"]) != source_images(copy, stack_path):
                raise ValueError(f"pinned Fast vendor snapshot differs: {copy['vendor']}")
            if (root / copy["owned"]).exists():
                raise ValueError(f"refusing to overwrite owned Fast copy: {copy['owned']}")
        created = []
        applied = []
        try:
            for copy in copies:
                destination = root / copy["owned"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copytree(root / copy["vendor"], destination)
                created.append(destination)
            with wired_manifests(root, stack_path) as paths:
                try:
                    for path in paths:
                        apply_patch(path, "--apply", root)
                        applied.append(path)
                except Exception:
                    for path in reversed(applied):
                        apply_patch(path, "--reverse", root)
                    raise
        except Exception:
            for destination in created:
                shutil.rmtree(destination)
            raise
        check_copies(root, wired=True, stack_path=stack_path)
    elif action == "--reverse":
        check_copies(root, wired=True, stack_path=stack_path)
        check_manifest_images(root, stack_path)
        with wired_manifests(root, stack_path) as paths:
            for path in reversed(paths):
                apply_patch(path, "--reverse", root)
        check_copies(root, wired=False, stack_path=stack_path)
        for copy in copies:
            shutil.rmtree(root / copy["owned"])
        print("Owned Fast source copies removed; Kit manifests restored exactly")
    else:
        check_copies(root, wired=True, stack_path=stack_path)
        check_manifest_images(root, stack_path)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--stack", type=Path, default=DAMAGE_STACK)
    mode = parser.add_mutually_exclusive_group(required=True)
    for option in ("apply", "check", "reverse"):
        mode.add_argument("--" + option, action="store_true")
    args = parser.parse_args()
    action = next("--" + key for key in ("apply", "check", "reverse") if getattr(args, key))
    try:
        operate(action, args.root.resolve(), args.stack.resolve())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")
