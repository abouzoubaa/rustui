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
                "print(json.dumps({'rustc': os.environ.get('RUSTC'), 'args': sys.argv[1:]}))\n",
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


if __name__ == "__main__":
    unittest.main()
