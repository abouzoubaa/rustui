#!/usr/bin/env python3
"""Apply the approved, same-version GPUI bidirectional input hit testing patch exactly."""

import argparse
import hashlib
import json
import os
import stat
import subprocess
import tempfile
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "patches/approved/gpui-bidi-input-hit-v1.json"
EXPECTED_FILES = {
    "vendor/gpui-pre-0.3.9/src/text_system/line_layout.rs",
    "vendor/gpui-pre-0.3.9/src/text_system/line.rs",
    "vendor/gpui-pre-0.3.9/src/platform.rs",
    "vendor/gpui-pre-web-0.3.9/src/run_replacements.rs",
    "vendor/gpui-pre-wgpu-0.3.9/src/cosmic_text_system.rs",
    "vendor/gpui-pre-apple-0.3.9/src/text_system.rs",
    "vendor/gpui-pre-windows-0.3.9/src/direct_write.rs",
    "vendor/gpui-base-0.7.3/src/text/inline.rs",
}
PACKAGES = {
    "gpui-pre-0.3.9": "gpui-pre",
    "gpui-pre-web-0.3.9": "gpui-pre-web",
    "gpui-pre-wgpu-0.3.9": "gpui-pre-wgpu",
    "gpui-pre-apple-0.3.9": "gpui-pre-apple",
    "gpui-pre-windows-0.3.9": "gpui-pre-windows",
    "gpui-base-0.7.3": "gpui-base",
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_manifest() -> dict:
    meta = json.loads(MANIFEST.read_text())
    if set(meta["images"]) != EXPECTED_FILES:
        raise ValueError("patch file set changed")
    patch = ROOT / meta["patch"]
    if patch != ROOT / "patches/approved/gpui-bidi-input-hit-v1.patch":
        raise ValueError("patch path changed")
    if digest(patch.read_bytes()) != meta["patch_sha256"]:
        raise ValueError("patch bytes changed")
    for name, expected in meta["preserved_checksum_files"].items():
        if digest((ROOT / name).read_bytes()) != expected:
            raise ValueError(f"upstream checksum metadata changed: {name}")
    for directory, package_name in PACKAGES.items():
        package = tomllib.loads((ROOT / "vendor" / directory / "Cargo.toml").read_text())[
            "package"
        ]
        if (package["name"], package["version"]) != (package_name, directory.rsplit("-", 1)[1]):
            raise ValueError(f"upstream package identity changed: {directory}")
    return meta


def current_images(meta: dict) -> tuple[str, dict[str, bytes]]:
    images = {name: (ROOT / name).read_bytes() for name in sorted(EXPECTED_FILES)}
    hashes = {name: digest(data) for name, data in images.items()}
    for state in ("pre", "post"):
        if all(hashes[name] == meta["images"][name][f"{state}_sha256"] for name in images):
            return state, images
    raise ValueError("source files are mixed or differ from both approved images")


def staged_images(meta: dict, before: dict[str, bytes], reverse: bool) -> dict[str, bytes]:
    with tempfile.TemporaryDirectory(prefix="rustui-bidi-patch-") as directory:
        stage = Path(directory)
        for name, data in before.items():
            path = stage / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        command = ["git", "apply", "--whitespace=nowarn"]
        if reverse:
            command.append("--reverse")
        patch = str(ROOT / meta["patch"])
        subprocess.run(command + ["--check", patch], cwd=stage, check=True)
        subprocess.run(command + [patch], cwd=stage, check=True)
        after = {name: (stage / name).read_bytes() for name in before}
    state = "pre" if reverse else "post"
    for name, data in after.items():
        if digest(data) != meta["images"][name][f"{state}_sha256"]:
            raise ValueError(f"unexpected {state} image: {name}")
    return after


def replace_images(before: dict[str, bytes], after: dict[str, bytes]) -> None:
    temporary: dict[str, Path] = {}
    replaced: list[str] = []
    try:
        for name, data in after.items():
            path = ROOT / name
            fd, temporary_name = tempfile.mkstemp(prefix=".rustui-bidi-", dir=path.parent)
            temp = Path(temporary_name)
            temporary[name] = temp
            with os.fdopen(fd, "wb") as output:
                os.fchmod(output.fileno(), stat.S_IMODE(path.stat().st_mode))
                output.write(data)
        if any((ROOT / name).read_bytes() != data for name, data in before.items()):
            raise ValueError("source changed while staging patch")
        for name, temp in temporary.items():
            os.replace(temp, ROOT / name)
            replaced.append(name)
    except Exception:
        for name in reversed(replaced):
            path = ROOT / name
            fd, temporary_name = tempfile.mkstemp(prefix=".rustui-bidi-rollback-", dir=path.parent)
            with os.fdopen(fd, "wb") as output:
                os.fchmod(output.fileno(), stat.S_IMODE(path.stat().st_mode))
                output.write(before[name])
            os.replace(temporary_name, path)
        raise
    finally:
        for temp in temporary.values():
            temp.unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group(required=True)
    for mode in ("check", "apply", "verify", "reverse"):
        modes.add_argument(f"--{mode}", action="store_true")
    args = parser.parse_args()
    mode = next(mode for mode in ("check", "apply", "verify", "reverse") if getattr(args, mode))
    meta = load_manifest()
    state, before = current_images(meta)
    if mode == "verify":
        if state != "post":
            raise ValueError("approved patch is not applied")
    elif mode == "check":
        staged_images(meta, before, state == "post")
    else:
        expected = "pre" if mode == "apply" else "post"
        if state != expected:
            raise ValueError(f"{mode} requires exact {expected} images")
        replace_images(before, staged_images(meta, before, mode == "reverse"))
    print(f"gpui bidi input hit: {mode} passed ({state} image)")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"gpui bidi input hit: {error}") from error
