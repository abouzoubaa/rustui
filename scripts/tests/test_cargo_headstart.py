from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LAUNCHER = ROOT / "scripts" / "cargo-headstart"


class CargoHeadstartTests(unittest.TestCase):
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
