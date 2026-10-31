#!/usr/bin/env python3
"""Check the proposed GPUI Components Slider accessible-name patch in isolation."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib


PATCH = Path("patches/proposed/gpui-component-slider-accessibility-label.patch")
PATCH_SHA256 = "9f187b1b830e8cda79f7e725ccb6c751b7cf2cb9644bc64b64213295a3d388d0"
PACKAGE = Path("vendor/gpui-component-0.7.3/Cargo.toml")
PACKAGE_ID = ("gpui-component", "0.7.3")
SOURCE = Path("vendor/gpui-component-0.7.3/src/slider.rs")
PREIMAGE_SHA256 = "94afa288c201ae24c67bb176d9825eef32d2b32d849810275b5051a6bcdb984d"
POSTIMAGE_SHA256 = "bb67bbd59904cb335cb45483029d58fbbc3a650d2e4199277fea4e59fe9fad97"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_invariants(source: str) -> None:
    invariants = (
        ("optional label defaults to unset", "accessibility_label: Option<SharedString>" in source
         and "accessibility_label: None," in source),
        ("builder accepts an accessible name", "pub fn accessibility_label(" in source
         and "self.accessibility_label = Some(label.into());" in source),
        ("name reaches the base slider's accessibility node",
         ".when_some(self.accessibility_label, |this, label| {" in source
         and "this.aria_label(label)" in source),
    )
    failures = [description for description, passed in invariants if not passed]
    if failures:
        raise ValueError("proposed patch invariants failed: " + "; ".join(failures))


def verify(root: Path) -> None:
    root = root.resolve()
    if sha256(root / PATCH) != PATCH_SHA256:
        raise ValueError("proposed patch SHA-256 mismatch")
    package = tomllib.loads((root / PACKAGE).read_text(encoding="utf-8"))["package"]
    if (package.get("name"), package.get("version")) != PACKAGE_ID:
        raise ValueError("source is not the pinned gpui-component 0.7.3 package")
    source = root / SOURCE
    if sha256(source) != PREIMAGE_SHA256:
        raise ValueError("Slider source is not the expected pinned preimage")

    with tempfile.TemporaryDirectory(prefix="rustui-gpui-slider-accessibility-patch-") as temp:
        stage = Path(temp)
        staged_source = stage / SOURCE
        staged_source.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, staged_source)
        result = subprocess.run(
            ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)],
            cwd=stage,
            capture_output=True,
            text=True,
            check=False,
        )
        output = result.stdout + result.stderr
        if result.returncode or re.search(r"offset|fuzz|FAILED", output, re.IGNORECASE):
            raise ValueError(f"exact isolated patch application failed: {output.strip()}")
        if sha256(staged_source) != POSTIMAGE_SHA256:
            raise ValueError("unexpected isolated Slider postimage hash")
        require_invariants(staged_source.read_text(encoding="utf-8"))

        rustfmt = shutil.which("rustfmt")
        if rustfmt is None:
            raise ValueError("rustfmt is required to check the isolated proposal")
        formatting = subprocess.run(
            [rustfmt, "--check", "--edition=2024", str(staged_source)],
            cwd=stage,
            capture_output=True,
            text=True,
            check=False,
        )
        if formatting.returncode:
            raise ValueError(
                f"isolated Rust formatting check failed: {formatting.stdout}{formatting.stderr}"
            )

    print(f"proposal patch SHA-256 verified: {PATCH_SHA256}")
    print("exact patch applied to a disposable Slider source copy; vendor files untouched")
    print("pinned source hashes and accessible-name storage/forwarding verified")
    print("rustfmt check passed on the disposable patched source")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        verify(args.root)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"error: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
