"""Compile the candidate's exact runtime selector and handoff state tests."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FrameSourceTests(unittest.TestCase):
    def test_runtime_selection_and_generation_safe_handoff(self):
        mac = (ROOT / 'crates/rustui-gpui-fast-macos/src/fast/frame_pacing.rs').read_text()
        apple = (ROOT / 'crates/rustui-gpui-fast-apple/src/fast/frame_pacing.rs').read_text()
        core = (ROOT / 'crates/rustui-gpui-fast/src/fast/frame_pacing.rs').read_text()
        selector_start = mac.rfind('#[derive(', 0, mac.index('pub(crate) enum Choice'))
        selector = mac[selector_start:mac.index('pub(crate) fn choose')]
        selector += mac[mac.index('#[cfg(test)]'):mac.index('pub(crate) struct LegacyFrames')]
        gate = 'use std::sync::atomic::{AtomicU64, Ordering};\n'
        gate += apple[apple.index('#[derive(Default)]'):apple.index('pub(crate) struct State')]
        gate += apple[apple.index('#[cfg(test)]'):]
        switches = 'use std::sync::OnceLock;\n'
        switches += core[core.index('pub fn fast_frame_ahead_enabled'):core.index('struct InputState')]
        switches += '\n#[test] fn new_switches_default_off(){assert!(!fast_frame_ahead_enabled());assert!(!fast_view_link_requested());assert!(!fast_force_legacy_link());}\n'
        env = dict(os.environ)
        for key in ('GPUI_FAST_FRAME_AHEAD', 'GPUI_FAST_VIEW_DISPLAY_LINK', 'GPUI_FAST_FORCE_LEGACY_LINK'):
            env.pop(key, None)
        with tempfile.TemporaryDirectory(prefix='rustui-g5-sources-') as directory:
            folder = Path(directory)
            for name, source in [('selector', selector), ('handoff', gate), ('switches', switches)]:
                rust = folder / (name + '.rs')
                binary = folder / name
                rust.write_text(source)
                subprocess.run(['rustup', 'run', '1.98.0', 'rustc', '--edition=2024', '--test', str(rust), '-o', str(binary)], cwd=ROOT, env=env, check=True)
                subprocess.run([str(binary)], cwd=ROOT, env=env, check=True)
                if name == "switches":
                    subprocess.run([str(binary)], cwd=ROOT, env=dict(env, GPUI_FAST_FRAME_AHEAD="1"), check=True)


if __name__ == '__main__':
    unittest.main()
