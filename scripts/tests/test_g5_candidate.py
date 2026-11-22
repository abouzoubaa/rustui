"""Compile the candidate's exact portable state logic and its own tests."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CandidateStateTests(unittest.TestCase):
    def test_exact_pool_and_scope_tests_and_default_off(self):
        apple = (ROOT / 'crates/rustui-gpui-fast-apple/src/fast/immediate_present.rs').read_text()
        core = (ROOT / 'crates/rustui-gpui-fast/src/fast/immediate_present.rs').read_text()
        capacity=(ROOT/'crates/rustui-gpui-fast-apple/src/fast/drawable_capacity.rs').read_text()
        pool = 'mod fast {pub mod drawable_capacity {'+capacity+'}}\n'+apple[apple.index('#[derive(Default)]'):apple.index('pub(crate) struct State')]
        tests = apple[apple.index('#[cfg(test)]'):]
        scope = core[core.index('thread_local! { static ACTIVE'):core.index('// Hold clock')]
        scope += core[core.index('// Hold clock'):core.index('// Per-window')]
        scope_tests = core[core.index('#[cfg(test)]'):]
        env = dict(os.environ)
        env.pop('GPUI_FAST_IMMEDIATE_PRESENT', None)
        env.pop('GPUI_FAST_IMMEDIATE_HOLD_MS', None)
        snippets = [('pool', 'use std::collections::HashMap;\n' + pool + tests),
                    ('scope', 'use std::{cell::Cell,marker::PhantomData,rc::Rc,sync::{OnceLock,Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};\n'
                     + scope + scope_tests
                     + '\n#[test] fn switch_defaults_off(){assert!(!fast_immediate_enabled());}\n#[test] fn hold_defaults_250_ms(){assert_eq!(hold_duration(),Duration::from_millis(250));}\n')]
        mac = (ROOT / 'crates/rustui-gpui-fast-macos/src/fast/immediate_present.rs').read_text()
        wake = mac[mac.index('#[derive(Default)]'):mac.index('pub(crate) fn frame_waker')]
        snippets.append(('wake', 'use std::cell::Cell;\n' + wake + mac[mac.index('#[cfg(test)]'):]))
        with tempfile.TemporaryDirectory(prefix='rustui-g5-state-') as temporary:
            folder = Path(temporary)
            for name, source in snippets:
                rust = folder / (name + '.rs')
                binary = folder / name
                rust.write_text(source)
                subprocess.run(['rustup', 'run', '1.98.0', 'rustc', '--edition=2024', '--test', str(rust), '-o', str(binary)], cwd=ROOT, env=env, check=True)
                subprocess.run([str(binary)], cwd=ROOT, env=env, check=True)

    def test_native_pixel_filter_emits_returns_to_old_versions(self):
        source = (ROOT / 'apps/latency-ref/pixel_capture.swift').read_text()
        transition = source[source.index('struct VersionTransitions'):source.index('final class VersionCapture')]
        with tempfile.TemporaryDirectory(prefix='rustui-g5-pixels-') as temporary:
            folder = Path(temporary)
            swift = folder / 'main.swift'
            binary = folder / 'transitions'
            swift.write_text(transition + '\nvar t=VersionTransitions()\n'
                             'precondition(t.observe(1));precondition(!t.observe(1));'
                             'precondition(t.observe(2));precondition(t.observe(1));'
                             'precondition(!t.observe(1));precondition(t.observe(0))\n')
            subprocess.run(['xcrun', 'swiftc', str(swift), '-o', str(binary)], cwd=ROOT, check=True)
            subprocess.run([str(binary)], cwd=ROOT, check=True)


if __name__ == '__main__':
    unittest.main()
