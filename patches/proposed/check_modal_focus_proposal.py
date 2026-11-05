#!/usr/bin/env python3
"""Check both proposed patches on disposable exact-image copies only."""
import hashlib
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
GROUPS = (
    ("gpui-component-modal-focus-v1.patch", "353ab857b3085288530fabf397a825d5b20828cc21b64314068ac256dd37fd03", 2, (
        ("vendor/gpui-component-0.7.3/src/root.rs", "root.rs", "c438aabc60f18252420a40e773e0aa4447fe89b00e27033c0ad7e5980b9c2749", "70eb3b61caf79896a09124271b23a8c67661d0c150d1899608710014d2e9a3b9"),
        ("vendor/gpui-component-0.7.3/src/window_ext.rs", "window_ext.rs", "697b8a6867d1f12085ac6c976568745f6ba4ca21f5a9727c7a74e30523ed77eb", "42f7d726334daf6dec12e46e1ef7cecab0cdccced6ba336c644edb0039d793df"),
    )),
    ("rustui-modal-focus-integration-v1.patch", "5fcfed26ceef92525e6b650385c685facd1615cfd39ff1f39cb62564700f8c8c", 1, (
        ("crates/rustui/src/runtime/window.rs", "crates/rustui/src/runtime/window.rs", "9899b606b34fae666b9a016773ef4d3584ca5c2d4d4a6decfccbdacd510efd85", "8d1e02253aaf0cf2497a15d73a417eead5729a678163e99af94689b186ed0274"),
    )),
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    for name, patch_hash, strip, files in GROUPS:
        patch = ROOT / "patches/proposed" / name
        require(digest(patch) == patch_hash, f"unexpected patch: {name}")
        with tempfile.TemporaryDirectory(prefix="rustui-modal-check-") as folder:
            stage = Path(folder)
            for source, target, pre, _ in files:
                require(digest(ROOT / source) == pre, f"unexpected source: {source}")
                dest = stage / target
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / source, dest)
            command = ["patch", "--batch", "--fuzz=0", f"-p{strip}", "-d", str(stage), "-i", str(patch)]
            for reverse in (False, True):
                result = subprocess.run(command + (["--reverse"] if reverse else []), check=True, capture_output=True, text=True)
                require("offset" not in result.stdout.lower(), f"patch offset: {name}")
                for _, target, pre, post in files:
                    require(digest(stage / target) == (pre if reverse else post), f"unexpected image: {target}")
        print(f"{name}: exact apply and reverse passed")


if __name__ == "__main__":
    main()
