from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "scripts" / "pinned-toolchain.sh"
SCRIPTS = (
    ROOT / "scripts" / "check.sh",
    ROOT / "scripts" / "smoke-playground.sh",
    ROOT / "scripts" / "perf-baseline.sh",
    ROOT / "scripts" / "verify-release.sh",
)


class PinnedToolchainTests(unittest.TestCase):
    def test_scripts_source_pinned_helper_and_use_its_rust_tools(self) -> None:
        for script in SCRIPTS:
            with self.subTest(script=script.name):
                content = script.read_text(encoding="utf-8")
                self.assertIn("pinned-toolchain.sh", content)
                self.assertNotRegex(content, r"rustup run [0-9]+\.[0-9]+\.[0-9]+")
                if script.name == "perf-baseline.sh":
                    self.assertIn("pinned_cargo", content)
                    self.assertIn("pinned_rustc", content)
                elif script.name == "verify-release.sh":
                    self.assertIn("pinned_cargo", content)
                    self.assertIn("pinned_rustc", content)
                else:
                    self.assertIn("pinned_cargo", content)

    def test_poisoned_compiler_environment_uses_rust_toolchain_channel(self) -> None:
        channel = next(
            line.split('"')[1]
            for line in (ROOT / "rust-toolchain.toml").read_text().splitlines()
            if line.strip().startswith("channel")
        )
        with tempfile.TemporaryDirectory(prefix="pinned rustup ") as temporary:
            directory = Path(temporary)
            log = directory / "rustup.jsonl"
            rustup = directory / "rustup"
            rustup.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, sys\n"
                "args = sys.argv[1:]\n"
                f"channel = {channel!r}\n"
                "if args[:1] == ['which']:\n"
                "    assert args[2:] == ['--toolchain', channel], args\n"
                "    print('/fake/' + channel + '/bin/' + args[1])\n"
                "elif args[:1] == ['run']:\n"
                "    assert args[1] == channel, args\n"
                "    assert os.environ.get('RUSTC') == '/fake/' + channel + '/bin/rustc'\n"
                "    assert os.environ.get('RUSTDOC') == '/fake/' + channel + '/bin/rustdoc'\n"
                f"    with open({str(log)!r}, 'a') as stream:\n"
                "        stream.write(json.dumps(args) + '\\n')\n"
                "    if args[2:4] == ['rustc', '--version']:\n"
                "        print('rustc ' + channel + ' (fake)')\n"
                "    elif args[2:4] == ['cargo', '--version']:\n"
                "        print('cargo for ' + channel)\n"
                "else:\n"
                "    raise SystemExit('unexpected rustup arguments: ' + repr(args))\n",
                encoding="utf-8",
            )
            rustup.chmod(0o755)
            env = os.environ.copy()
            env["PATH"] = f"{directory}:{env['PATH']}"
            env["RUSTC"] = "/opt/homebrew/bin/rustc"
            env["RUSTDOC"] = "/opt/homebrew/bin/rustdoc"
            env["repository_root"] = str(ROOT)
            command = (
                f'. "{HELPER}"; '
                'printf "overrides=%s,%s\\n" "${RUSTC+x}" "${RUSTDOC+x}"; '
                'pinned_rustc --version; pinned_cargo --version'
            )
            result = subprocess.run(
                ["sh", "-c", command], cwd=ROOT, env=env,
                capture_output=True, text=True, check=True,
            )
            self.assertEqual(
                result.stdout.splitlines(),
                ["overrides=x,x", f"rustc {channel} (fake)", f"cargo for {channel}"],
            )
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertEqual(calls, [["run", channel, "rustc", "--version"],
                                     ["run", channel, "cargo", "--version"]])


if __name__ == "__main__":
    unittest.main()
