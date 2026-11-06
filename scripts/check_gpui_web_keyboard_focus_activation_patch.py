#!/usr/bin/env python3
"""Check the proposed GPUI Web keyboard patch on disposable source copies."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib


PATCH = Path("patches/proposed/gpui-pre-web-keyboard-focus-activation-v3.patch")
PATCH_SHA256 = "df97bcbf43490e1e60e582da87b5bd63ebc2eb275e662f5c49d622f77b7de59a"
PACKAGE = Path("vendor/gpui-pre-web-0.3.9/Cargo.toml")
PACKAGE_ID = ("gpui-pre-web", "0.3.9")
FILES = {
    "vendor/gpui-pre-web-0.3.9/src/window.rs": (
        "eda543e585912fcb012e2a3559d8fb2e9ad7570ba58746747af380d044085cb8",
        "3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8",
    ),
    "vendor/gpui-pre-web-0.3.9/src/events.rs": (
        "9fdfc3695a93083fe881153dcc66356df00eb09761190664cc093520bc3ae001",
        "3dca61104bce15bffdd7d9155af769dc166c68508fb6458b4952261b2c3244cb",
    ),
}


def sha256_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def method_body(source: str, method: str, next_method: str) -> str:
    start = source.index(method)
    end = source.index(next_method, start)
    return source[start:end]


def require_invariants(window: str, events: str) -> None:
    pointer_down = method_body(events, "fn register_pointer_down", "fn pointer_targets_text_input")
    key_down = method_body(events, "fn register_key_down", "fn register_key_up")
    key_up = method_body(events, "fn register_key_up", "/// Imports IME edits")
    before_input = method_body(events, "fn register_before_input", "fn register_paste")
    text_input = method_body(events, "fn register_input", "/// Imports IME-driven selection moves")
    composition_start = method_body(events, "fn register_composition_start", "fn register_composition_update")
    composition_update = method_body(events, "fn register_composition_update", "fn register_composition_end")
    composition_end = method_body(events, "fn register_composition_end", "fn register_focus")
    target_guard = method_body(events, "fn is_keyboard_event_target", "fn listen_non_passive")

    invariants = (
        ("canvas is keyboard tabbable", "canvas.set_tab_index(0);" in window),
        (
            "pointer input continues to focus the IME mirror",
            'self.listen("pointerdown"' in pointer_down and "this.ime_mirror.focus();" in pointer_down,
        ),
        (
            "keydown is observed once at the bubbling window target",
            "EventListenerHandle::add(" in key_down
            and "self.browser_window.as_ref()" in key_down
            and '"keydown"' in key_down
            and "self.listen_input(\"keydown\"" not in key_down
            and 'self.listen("keydown"' not in key_down,
        ),
        (
            "keyup is observed once at the bubbling window target",
            "EventListenerHandle::add(" in key_up
            and "self.browser_window.as_ref()" in key_up
            and '"keyup"' in key_up
            and "self.listen_input(\"keyup\"" not in key_up
            and 'self.listen("keyup"' not in key_up,
        ),
        (
            "keyboard routing accepts only canvas or IME mirror targets",
            "event.target().is_some_and" in target_guard
            and "self.canvas.as_ref()" in target_guard
            and "self.ime_mirror.event_target()" in target_guard
            and "is_keyboard_event_target(&event)" in key_down
            and "is_keyboard_event_target(&event)" in key_up,
        ),
        (
            "keyboard events still dispatch GPUI keydown and keyup",
            "PlatformInput::KeyDown(KeyDownEvent" in key_down
            and "PlatformInput::KeyUp(KeyUpEvent" in key_up,
        ),
        (
            "beforeinput and input remain attached to the IME mirror",
            'self.listen_input("beforeinput"' in before_input
            and 'self.listen_input("input"' in text_input,
        ),
        (
            "composition start, update, and end remain attached to the IME mirror",
            all(
                'self.listen_input("composition' in body
                for body in (composition_start, composition_update, composition_end)
            ),
        ),
    )
    failures = [description for description, satisfied in invariants if not satisfied]
    if failures:
        raise ValueError("proposed patch invariants failed: " + "; ".join(failures))


def verify(root: Path) -> None:
    root = root.resolve()
    if sha256(root / PATCH) != PATCH_SHA256:
        raise ValueError("proposed patch SHA-256 mismatch")

    package = tomllib.loads((root / PACKAGE).read_text(encoding="utf-8"))["package"]
    if (package.get("name"), package.get("version")) != PACKAGE_ID:
        raise ValueError("source is not the pinned gpui-pre-web 0.3.9 package")

    originals: dict[str, bytes] = {}
    for relative, (preimage, _) in FILES.items():
        source = root / relative
        if sha256(source) != preimage:
            raise ValueError(f"unexpected pinned source hash: {relative}")
        originals[relative] = source.read_bytes()

    with tempfile.TemporaryDirectory(prefix="rustui-gpui-web-keyboard-patch-") as temp:
        stage = Path(temp)
        for relative, contents in originals.items():
            staged = stage / relative
            staged.parent.mkdir(parents=True, exist_ok=True)
            staged.write_bytes(contents)

        result = subprocess.run(
            ["patch", "--batch", "--fuzz=0", "-p1", "-i", str(root / PATCH)],
            cwd=stage,
            capture_output=True,
            text=True,
            check=False,
        )
        output = result.stdout + result.stderr
        if result.returncode or re.search(r"offset|fuzz|FAILED", output, re.I):
            raise ValueError(f"exact isolated patch application failed: {output.strip()}")

        for relative, (_, postimage) in FILES.items():
            staged = stage / relative
            actual = sha256(staged)
            if actual != postimage:
                raise ValueError(f"unexpected patched source hash {relative}: {actual}")

        window = (stage / next(iter(FILES))).read_text(encoding="utf-8")
        events_relative = list(FILES)[1]
        events = (stage / events_relative).read_text(encoding="utf-8")
        require_invariants(window, events)

        rustfmt = shutil.which("rustfmt")
        if rustfmt is None:
            raise ValueError("rustfmt is required to check the isolated proposal")
        formatting = subprocess.run(
            [
                rustfmt,
                "--check",
                "--edition=2024",
                str(stage / list(FILES)[0]),
                str(stage / events_relative),
            ],
            cwd=stage,
            capture_output=True,
            text=True,
            check=False,
        )
        if formatting.returncode:
            raise ValueError(
                f"isolated rustfmt check failed: {formatting.stdout}{formatting.stderr}"
            )

    print(f"proposal patch SHA-256 verified: {PATCH_SHA256}")
    print("exact patch applied to disposable source copies; vendor files untouched")
    print("patched source hashes and focus, keyboard, text, and IME routing invariants verified")
    print("rustfmt check passed on disposable patched Rust sources")


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
