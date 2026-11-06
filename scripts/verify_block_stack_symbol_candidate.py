#!/usr/bin/env python3
"""Verify the isolated block 0.1.6 stack-symbol candidate on macOS."""
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path("vendor/block-0.1.6/src/lib.rs")
PATCH = Path("patches/proposed/block-0.1.6-stack-symbol.patch")
PRE = "eb31678adf63b53109d9b94eba23699fd5f9ebfdb950f6e1a57ad51bb6a146fa"
EXPECTED = {
    Path("vendor/block-0.1.6/.cargo-checksum.json"): "436ff3319faf156bd9a969481c9b0bf0cbebe3b22719c1353a3513447bab4fbf",
    PATCH: "d0fc7338cf3e8cc97477157e1cff3719257566105909a2a3f2af788c03e5fac6",
    Path("patches/proposed/block-stack-symbol-probe.rs"): "e7af5377afaa2eb2fdf5be3e25a4d2e1787dcac7faa20146cf645109c6c3d469",
    Path("patches/proposed/block-stack-symbol-ffi.c"): "cfc67b55635965f7dcf61f260d78e737a903386c26a9471aeb057f31096eb8a7",
}
POST = "798f4b286defc8fa722247a4b6c031d2a9b5144fc916e2e5c4cbb0ed85c8b4dd"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(root):
    if sys.platform != "darwin":
        raise ValueError("candidate FFI probe requires macOS")
    if digest(root / SOURCE) not in (PRE, POST):
        raise ValueError(f"unexpected input hash: {SOURCE}")
    for path, expected in EXPECTED.items():
        if digest(root / path) != expected:
            raise ValueError(f"unexpected input hash: {path}")
    host = next(line.split(": ", 1)[1] for line in subprocess.check_output(["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
    headstart = Path(os.environ.get("RUSTUI_HEADSTART_ROOT", Path.home() / ".local/opt/headstart"))
    rustc = headstart / "rustc/build" / host / "stage1/bin/rustc"
    if not rustc.is_file():
        raise ValueError("Headstart rustc is missing")
    with tempfile.TemporaryDirectory(prefix="rustui-block-ffi-") as folder:
        work = Path(folder)
        staged = work / SOURCE
        staged.parent.mkdir(parents=True)
        shutil.copyfile(root / SOURCE, staged)
        patch = str(root / PATCH)
        def apply(*args):
            subprocess.run(["git", "apply", *args, patch], cwd=work, check=True)
        if digest(staged) == POST:
            apply("--reverse")
        if digest(staged) != PRE:
            raise ValueError("candidate preimage mismatch")
        original = staged.read_bytes()
        apply("--check")
        apply()
        if digest(staged) != POST:
            raise ValueError("candidate postimage mismatch")
        apply("--reverse")
        if staged.read_bytes() != original:
            raise ValueError("candidate reversal mismatch")
        apply()
        subprocess.run(["clang", "-fblocks", "-c", str(root / "patches/proposed/block-stack-symbol-ffi.c"), "-o", str(work / "ffi.o")], check=True)
        subprocess.run([str(rustc), "--crate-name", "block", "--crate-type", "rlib", "--edition", "2015", "-Zmetadata-crate-hash=no", "-D", "uninhabited_static", str(staged), "-o", str(work / "libblock.rlib")], check=True)
        subprocess.run([str(rustc), "--edition", "2015", "-Zmetadata-crate-hash=no", "--extern", f"block={work}/libblock.rlib", "-C", f"link-arg={work}/ffi.o", str(root / "patches/proposed/block-stack-symbol-probe.rs"), "-o", str(work / "probe")], check=True)
        subprocess.run([str(work / "probe")], check=True)
    print("block candidate replay and macOS FFI probe passed")


if __name__ == "__main__":
    try:
        verify(ROOT)
    except (OSError, ValueError, StopIteration, subprocess.CalledProcessError) as error:
        print(f"block candidate verification failed: {error}", file=sys.stderr)
        sys.exit(1)
