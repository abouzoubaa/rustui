#!/usr/bin/env python3
"""Exercise the real Kit editor against a complete Metal redraw on the Mac main thread."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[1]


def manifest_text():
    def quote(value):
        return json.dumps(str(value))

    manifest = [
        "[workspace]",
        "[package]",
        'name = "rustui-editor-damage-oracle"',
        'version = "0.0.0"',
        'edition = "2024"',
        "[[bin]]",
        'name = "editor-damage-oracle"',
        f"path = {quote(ROOT / 'apps/latency-ref/g_r2e_editor_oracle.rs')}",
        "[dependencies]",
    ]
    for name, package, directory, features in (
        ("gpui", "gpui-fast", "rustui-gpui-fast", ["test-support"]),
        ("gpui_platform", "gpui-fast-platform", "rustui-gpui-fast-platform",
         ["test-support", "runtime_shaders"]),
        ("gpui_base", "gpui-base", "rustui-gpui-base", ["gpui-fast"]),
    ):
        manifest.append(
            f"{name} = {{package = {quote(package)}, path = {quote(ROOT/'crates'/directory)}, "
            f"features = {json.dumps(features)}}}"
        )
    manifest.extend(['[profile.dev.package."gpui-fast-apple"]', "opt-level = 2", "[patch.crates-io]"])
    patches = tomllib.loads((ROOT / "Cargo.toml").read_text())["patch"]["crates-io"]
    for package, spec in patches.items():
        manifest.append(f"{quote(package)} = {{path = {quote(ROOT / spec['path'])}}}")
    return "\n".join(manifest) + "\n"


def main():
    fixture = ROOT / "target/headstart/g-r2e/editor-fixture"
    fixture.mkdir(parents=True, exist_ok=True)
    (fixture / "Cargo.toml").write_text(manifest_text())
    (fixture / "Cargo.lock").write_bytes((ROOT / "Cargo.lock").read_bytes())
    target = Path.home() / ".cache/rustui-target"
    env = dict(os.environ, RUSTUI_DAMAGE="1", RUSTUI_DAMAGE_VERIFY="1",
               RUSTUI_DAMAGE_DECLARED="1", RUSTUI_G_R2_EDITOR_ORACLE="1",
               RUSTUI_G_R2_VIEW_SIGNATURES="1", RUSTUI_G5_PREPAINT_REUSE="1",
               CARGO_TARGET_DIR=str(target))
    subprocess.run(
        ["bash", "-c", '. scripts/pinned-toolchain.sh; pinned_cargo build --offline --manifest-path "$1"',
         "oracle", str(fixture / "Cargo.toml")], env=env, cwd=ROOT, check=True,
    )
    for case in ("edit", "newline", "paste", "selection", "caret"):
        subprocess.run([str(target / "debug/editor-damage-oracle"), case, *sys.argv[1:]],
                       env=env, check=True, cwd=ROOT)


if __name__ == "__main__":
    main()
