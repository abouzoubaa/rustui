from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LAUNCHER = ROOT / "scripts" / "cargo-headstart"


class CargoHeadstartTests(unittest.TestCase):
    def test_tool_rustdoc_restores_library_path_and_preserves_caller_override(self) -> None:
        with tempfile.TemporaryDirectory(prefix="headstart docs ") as temporary:
            root = Path(temporary)
            host = re.search(r"^host: (.+)$", subprocess.check_output(
                ["rustc", "-vV"], text=True), re.MULTILINE)[1]
            stage = root / "rustc/build" / host
            rustc = stage / "stage1/bin/rustc"
            rustc.parent.mkdir(parents=True)
            rustc.write_text("#!/bin/sh\nexit 0\n")
            rustc.chmod(0o755)
            rustdoc = stage / "stage1-tools-bin/rustdoc-tool-binary"
            rustdoc.parent.mkdir(parents=True)
            rustdoc.write_text(
                f"#!{sys.executable}\nimport json, os, sys\n"
                "print(json.dumps({'library': os.environ.get('DYLD_LIBRARY_PATH'), 'args': sys.argv[1:]}))\n"
            )
            rustdoc.chmod(0o755)
            caller = root / "caller-rustdoc"
            caller.write_text("#!/bin/sh\nprintf '%s\\n' '{\"caller\": true}'\n")
            caller.chmod(0o755)
            cargo = root / "cargo/target/release/cargo"
            cargo.parent.mkdir(parents=True)
            cargo.write_text(
                "#!/usr/bin/env python3\nimport os, subprocess\n"
                "env = os.environ.copy()\nenv.pop('DYLD_LIBRARY_PATH', None)\n"
                "print(subprocess.check_output([env['RUSTDOC'], 'space argument'], env=env, text=True))\n"
            )
            cargo.chmod(0o755)
            env = os.environ.copy()
            for name in ("RUSTC", "RUSTDOC", "DYLD_LIBRARY_PATH"):
                env.pop(name, None)
            env["RUSTUI_HEADSTART_ROOT"] = str(root)
            env["CARGO_TARGET_DIR"] = str(root / "target")
            result = subprocess.run([str(LAUNCHER), "test", "--doc"], env=env,
                                    capture_output=True, text=True, check=True)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["library"], str(stage / "stage1/lib"))
            self.assertEqual(payload["args"], ["space argument"])
            env["RUSTDOC"] = str(caller)
            result = subprocess.run([str(LAUNCHER), "test", "--doc"], env=env,
                                    capture_output=True, text=True, check=True)
            self.assertEqual(json.loads(result.stdout), {"caller": True})

    def test_explicit_rustc_is_preserved_and_default_stays_headstart(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            headstart = base / "headstart"
            cargo = headstart / "cargo" / "target" / "release" / "cargo"
            cargo.parent.mkdir(parents=True)
            cargo.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, sys\n"
                "print(json.dumps({'rustc': os.environ.get('RUSTC'), 'args': sys.argv[1:], 'encoded': os.environ.get('CARGO_ENCODED_RUSTFLAGS'), 'rustflags': os.environ.get('RUSTFLAGS')}))\n",
                encoding="utf-8",
            )
            cargo.chmod(0o755)

            host = re.search(
                r"^host: (.+)$",
                subprocess.check_output(["rustc", "-vV"], text=True),
                re.MULTILINE,
            )
            assert host is not None
            default_rustc = headstart / "rustc" / "build" / host[1] / "stage1" / "bin" / "rustc"
            default_rustc.parent.mkdir(parents=True)
            default_rustc.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            default_rustc.chmod(0o755)

            custom_rustc = base / "custom-rustc"
            custom_rustc.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            custom_rustc.chmod(0o755)

            env = os.environ.copy()
            env["RUSTUI_HEADSTART_ROOT"] = str(headstart)
            env["CARGO_TARGET_DIR"] = str(base / "target")
            env.pop("CARGO_ENCODED_RUSTFLAGS", None)
            env.pop("RUSTFLAGS", None)

            for requested_rustc, expected_rustc in (
                (str(custom_rustc), str(custom_rustc)),
                (None, str(default_rustc)),
            ):
                if requested_rustc is None:
                    env.pop("RUSTC", None)
                else:
                    env["RUSTC"] = requested_rustc
                result = subprocess.run(
                    [str(LAUNCHER), "check", "-p", "rustui"],
                    env=env,
                    text=True,
                    capture_output=True,
                    check=True,
                )
                payload = json.loads(result.stdout)
                self.assertEqual(payload["rustc"], expected_rustc)
                self.assertIn("-Zheadstart", payload["args"])
                self.assertEqual(payload["args"][-3:], ["check", "-p", "rustui"])


            # Opt-in release remapping must retain caller flag precedence and
            # one encoded argument for paths containing spaces.
            env["RUSTUI_REMAP_BUILD_PATHS"] = "1"
            env["CARGO_TARGET_DIR"] = str(base / "repro target with spaces")
            env["CARGO_HOME"] = str(base / "cargo home with spaces")
            mappings = [
                "--remap-path-prefix=" + str(ROOT.resolve()) + "=/rustui/src",
                "--remap-path-prefix=" + str(Path(env["CARGO_TARGET_DIR"]).resolve()) + "=/rustui/build",
                "--remap-path-prefix=" + str(Path(env["CARGO_HOME"]).resolve()) + "=/rustui/cargo",
            ]
            for encoded, raw, expected in (
                ("--cfg\x1fencoded_caller", "--cfg ignored_raw", ["--cfg", "encoded_caller"]),
                (None, "-C opt-level=2", ["-C", "opt-level=2"]),
                ("", "--cfg ignored_raw", []),
            ):
                if encoded is None:
                    env.pop("CARGO_ENCODED_RUSTFLAGS", None)
                else:
                    env["CARGO_ENCODED_RUSTFLAGS"] = encoded
                env["RUSTFLAGS"] = raw
                result = subprocess.run([str(LAUNCHER), "build", "--release"], env=env,
                                        text=True, capture_output=True, check=True)
                payload = json.loads(result.stdout)
                self.assertEqual(payload["encoded"].split("\x1f"),
                                 expected + mappings + ["-Zmetadata-crate-hash=no"])
                self.assertEqual(payload["rustflags"], raw)
            env["RUSTUI_REMAP_BUILD_PATHS"] = "2"
            result = subprocess.run([str(LAUNCHER), "check"], env=env,
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn("must be 0 or 1", result.stderr)


    def test_macos_popup_patch_wraps_development_cargo_and_preserves_outer_owner(self) -> None:
        with tempfile.TemporaryDirectory(prefix="headstart popup ") as temporary:
            root = Path(temporary)
            scripts = root / "scripts"
            scripts.mkdir()
            launcher = scripts / "cargo-headstart"
            launcher.write_bytes(LAUNCHER.read_bytes())
            launcher.chmod(0o755)
            helper = scripts / "apply_gpui_macos_anchored_popup_patch.py"
            helper.write_text(
                "import os, sys\n"
                "with open(os.environ['PATCH_LOG'], 'a') as log:\n"
                "    log.write('reverse\\n' if '--reverse' in sys.argv else 'apply\\n')\n"
                "print('reversed' if '--reverse' in sys.argv else "
                "os.environ.get('PATCH_APPLY_STATE', 'applied'))\n",
                encoding="utf-8",
            )
            (root / "vendor/gpui-pre-macos-0.3.9/src").mkdir(parents=True)
            fake_bin = root / "fake-bin"
            fake_bin.mkdir()
            uname = fake_bin / "uname"
            uname.write_text("#!/bin/sh\nprintf '%s\\n' Darwin\n", encoding="utf-8")
            uname.chmod(0o755)

            host = re.search(
                r"^host: (.+)$", subprocess.check_output(["rustc", "-vV"], text=True), re.MULTILINE
            )[1]
            headstart = root / "headstart"
            rustc = headstart / "rustc/build" / host / "stage1/bin/rustc"
            rustc.parent.mkdir(parents=True)
            rustc.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            rustc.chmod(0o755)
            cargo = headstart / "cargo/target/release/cargo"
            cargo.parent.mkdir(parents=True)
            cargo.write_text("#!/bin/sh\nexit \"${CARGO_FAKE_EXIT:-0}\"\n", encoding="utf-8")
            cargo.chmod(0o755)

            log = root / "patch.log"
            env = os.environ.copy()
            for name in ("RUSTC", "RUSTDOC", "DYLD_LIBRARY_PATH"):
                env.pop(name, None)
            env.update(
                RUSTUI_HEADSTART_ROOT=str(headstart),
                CARGO_TARGET_DIR=str(root / "target"),
                PATH=str(fake_bin) + os.pathsep + env["PATH"],
                PATCH_LOG=str(log),
            )

            result = subprocess.run([str(launcher), "check"], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(log.read_text(encoding="utf-8").splitlines(), ["apply", "reverse"])

            log.write_text("", encoding="utf-8")
            env["PATCH_APPLY_STATE"] = "already-applied"
            result = subprocess.run([str(launcher), "check"], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(log.read_text(encoding="utf-8").splitlines(), ["apply"])

            log.write_text("", encoding="utf-8")
            env.pop("PATCH_APPLY_STATE")
            env["CARGO_FAKE_EXIT"] = "23"
            result = subprocess.run([str(launcher), "check"], env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 23)
            self.assertEqual(log.read_text(encoding="utf-8").splitlines(), ["apply", "reverse"])


if __name__ == "__main__":
    unittest.main()
