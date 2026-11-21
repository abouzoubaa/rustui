"""Physical display correlation must not substitute submits or overwritten states."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from g0_latency import summarize_latency


def record(event, version, mono, first=None, window='w'):
    row = dict(event=event, version=version, first_visible=version if first is None else first,
               mono_ms=mono, epoch_ms=1000+mono, window=window, scene='typing')
    if event == 'presented':
        row.update(presented_time=100+mono/1000, valid=True)
    return row


class LatencyTests(unittest.TestCase):
    def summarize(self, rows, scene='typing', start=0, end=2000):
        for row in rows:
            row['scene'] = scene
        return summarize_latency(rows, scene, start, end, True)

    def test_disabled_and_idle_do_not_invent_zero_delay(self):
        for scene, enabled in [('typing', False), ('idle', True)]:
            result = summarize_latency([], scene, 0, 1, enabled)
            self.assertIsNone(result['median_ms'])
            self.assertEqual(result['samples'], 0)

    def test_earliest_actual_display_wins_despite_callback_order_and_submit(self):
        rows = [record('input', 1, 10), record('paint', 1, 11),
                record('submitted', 1, 12), record('presented', 1, 30),
                record('presented', 1, 20), record('input', 2, 22),
                record('paint', 2, 24), record('presented', 2, 29)]
        result = self.summarize(rows)
        self.assertEqual((result['median_ms'], result['p95_ms'], result['max_ms']), (8.5, 10, 10))
        self.assertEqual(result['delays_over_8_3_ms'], 1)

    def test_overwritten_unpainted_state_is_not_charged_to_a_later_version(self):
        result = self.summarize([record('input', 1, 1), record('input', 2, 2),
                                 record('paint', 2, 3), record('presented', 2, 5)])
        self.assertEqual((result['samples'], result['coalesced_unpainted']), (1, 1))
        self.assertEqual(result['median_ms'], 3)

    def test_visible_log_range_can_show_two_appends_in_one_drawable(self):
        result = self.summarize([record('input', 1, 1), record('input', 2, 2),
                                 record('paint', 2, 3, first=1),
                                 record('presented', 2, 6, first=1)], scene='log')
        self.assertEqual((result['samples'], result['median_ms']), (2, 4.5))

    def test_unpainted_drawable_tag_is_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'never painted'):
            self.summarize([record('input', 1, 1), record('presented', 1, 5)])

    def test_zero_host_time_cannot_stand_in_for_display(self):
        invalid = record('presented', 1, 4)
        invalid.update(presented_time=0, valid=False)
        result = self.summarize([record('input', 1, 1), record('paint', 1, 2), invalid,
                                 record('presented', 1, 11)])
        self.assertEqual((result['median_ms'], result['invalid_presented_callbacks']), (10, 1))

    def test_input_window_keeps_its_first_display_after_end_marker(self):
        rows = [record('input', 0, 1), record('input', 1, 10), record('paint', 1, 11),
                record('presented', 1, 17)]
        result = self.summarize(rows, start=1005, end=1011)
        self.assertEqual((result['inputs'], result['median_ms']), (1, 7))

    def test_real_core_recorder_is_off_by_default_and_tags_only_after_paint(self):
        source = (ROOT/'vendor/gpui-fast-0.1.14/src/fast/input_latency.rs').read_text()
        stub = '''type WindowId = u64;
struct Window; struct Handle;
impl Window { fn window_handle(&self) -> Handle { Handle } }
impl Handle { fn window_id(&self) -> u64 { 7 } }
mod tracing {
'''
        harness = stub+source+'''
}
fn main() {
    let stamp=Window.fast_measure_input("typing", 1, 1);
    tracing::painted(7);
    { let _scope=tracing::present_scope(7); assert!(tracing::fast_present_tag().is_none()); }
    Window.fast_input_handled(stamp);
    tracing::painted(7);
    let scope=tracing::present_scope(7);
    if let Some(tag)=tracing::fast_present_tag() {
        std::thread::sleep(std::time::Duration::from_millis(3));
        tracing::fast_frame_presented(tag, 100.0, 100.001);
    }
    drop(scope);
    assert!(tracing::fast_present_tag().is_none());
}
'''
        channel = next(line.split('"')[1] for line in (ROOT/'rust-toolchain.toml').read_text().splitlines() if line.strip().startswith('channel'))
        with tempfile.TemporaryDirectory() as tmp:
            rust, binary = Path(tmp)/'trace.rs', Path(tmp)/'trace'
            rust.write_text(harness)
            build = subprocess.run(['rustup','run',channel,'rustc','--edition','2024',str(rust),'-o',str(binary)],capture_output=True,text=True)
            self.assertEqual(build.returncode,0,build.stderr)
            env = dict(os.environ,RUSTUI_G0_LATENCY='0',RUSTUI_FRAME_LOG='0')
            disabled = subprocess.run([str(binary)],env=env,capture_output=True,text=True)
            self.assertEqual(disabled.returncode,0,disabled.stderr)
            self.assertEqual(disabled.stderr,'')
            env['RUSTUI_G0_LATENCY']='1'
            enabled = subprocess.run([str(binary)],env=env,capture_output=True,text=True)
            self.assertEqual(enabled.returncode,0,enabled.stderr)
            rows=[json.loads(line.removeprefix('RUSTUI_INPUT_LOG ')) for line in enabled.stderr.splitlines()]
            self.assertEqual([r['event'] for r in rows],['input','paint','presented'])
            self.assertEqual(rows[-1]['presented_time'],100.)
            self.assertTrue(rows[-1]['valid'])
            self.assertGreaterEqual(rows[-1]['mono_ms'],rows[1]['mono_ms'])
