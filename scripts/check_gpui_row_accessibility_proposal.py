#!/usr/bin/env python3
"""Check the proposed row-accessibility series in disposable copies only."""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

MANIFEST = Path("patches/proposed/gpui-row-accessibility-series-v1.json")
MANIFEST_SHA256 = "ed0a205650efdb4459f01703abc259ee0dcf618e1e03efede437956ccecbba3e"
SOURCES = {
    "gpui-pre-0.3.9": ("gpui-pre", "0.3.9", "src/elements/div.rs"),
    "gpui-component-0.7.3": ("gpui-component", "0.7.3", "src/list/list.rs"),
}


_POPUP_SPEC = importlib.util.spec_from_file_location(
    "select_popup_rtl", Path(__file__).with_name("apply_gpui_select_popup_rtl_patch.py")
)
POPUP = importlib.util.module_from_spec(_POPUP_SPEC)
_POPUP_SPEC.loader.exec_module(POPUP)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(root: Path) -> dict:
    root = root.resolve()
    if digest(root / MANIFEST) != MANIFEST_SHA256:
        raise ValueError("proposal metadata hash mismatch")
    manifest = json.loads((root / MANIFEST).read_text())
    records = manifest["sources"]
    if [r["package_directory"] for r in records] != list(SOURCES):
        raise ValueError("unexpected proposal source set")
    before = {}
    popup_applied = False
    for record in records:
        package = record["package_directory"]
        name, version, relative = SOURCES[package]
        directory = root / "vendor" / package
        if record["source"] != f"vendor/{package}/{relative}":
            raise ValueError("unexpected proposal source path")
        identity = tomllib.loads((directory / "Cargo.toml").read_text())["package"]
        if (identity["name"], identity["version"]) != (name, version):
            raise ValueError("changed upstream package identity")
        for path, expected in [
            (directory / "Cargo.toml", record["cargo_manifest_sha256"]),
            (directory / ".cargo-checksum.json", record["checksum_metadata_sha256"]),
            (root / record["source"], record["pre_sha256"]),
            (root / record["patch"], record["patch_sha256"]),
        ]:
            if path.is_symlink() or not path.resolve().is_relative_to(root):
                raise ValueError("proposal input escapes the checkout")
            actual = digest(path)
            if actual != expected:
                # Accept only exact, independently verified composed images.
                rtl_image = POPUP.IMAGES.get(Path(record["source"]))
                if (path == root / record["source"] and rtl_image
                        and (expected, actual) == rtl_image):
                    with contextlib.redirect_stdout(io.StringIO()):
                        POPUP.run(root, root, "--verify")
                    popup_applied = True
                elif not (package == "gpui-component-0.7.3"
                        and path == directory / "Cargo.toml"
                        and expected == "bc3d39631970412dd2abbe1d5e34b0a0dbc65e6e995f13f3809895cda0ecc3c8"
                        and actual == "192f4156aee8738567c748d6b1c5bc44c80e4ba5369a41e5349e370d83b35026"):
                    raise ValueError(f"proposal input hash mismatch: {path.relative_to(root)}")
            before[path] = actual
        if record["vcs_metadata_sha256"] is not None:
            path = directory / ".cargo_vcs_info.json"
            if path.is_symlink() or not path.resolve().is_relative_to(root):
                raise ValueError("upstream VCS metadata escapes the checkout")
            if digest(path) != record["vcs_metadata_sha256"]:
                raise ValueError("changed upstream VCS metadata")
            before[path] = record["vcs_metadata_sha256"]
    with tempfile.TemporaryDirectory(prefix="rustui-row-a11y-check-") as temporary:
        stage = Path(temporary)
        for record in records:
            source = stage / record["source"]
            source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(root / record["source"], source)
        if popup_applied:
            for relative in POPUP.IMAGES:
                target = stage / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(root / relative, target)
                before[root / relative] = digest(root / relative)
            before[root / POPUP.PATCH] = digest(root / POPUP.PATCH)
            subprocess.run(
                ["git", "apply", "--reverse", "--whitespace=nowarn", str(root / POPUP.PATCH)],
                cwd=stage, check=True, capture_output=True,
            )
        for reverse, ordered in [(False, records), (True, list(reversed(records)))]:
            for record in ordered:
                command = ["git", "apply", "--whitespace=error-all"]
                if reverse:
                    command.append("--reverse")
                patch = str(root / record["patch"])
                for check in [True, False]:
                    result = subprocess.run(
                        command + (["--check"] if check else []) + ["--", patch],
                        cwd=stage, capture_output=True, text=True, check=False,
                    )
                    if result.returncode:
                        raise ValueError(f"isolated patch replay failed: {result.stderr.strip()}")
                expected = record["pre_sha256" if reverse else "post_sha256"]
                if digest(stage / record["source"]) != expected:
                    raise ValueError("isolated patch image hash mismatch")
    if any(digest(path) != expected for path, expected in before.items()):
        raise ValueError("selected source or provenance changed during verification")
    return {"status": "pass", "sources": len(records), "apply_reverse": "exact", "selected_sources": "unchanged"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        print(json.dumps(verify(args.root), sort_keys=True))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
