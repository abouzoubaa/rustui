"""Run the candidate's actual surface state tests and check draw-first queue ordering."""
from pathlib import Path
import os,subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[2]
class DrawFirstTests(unittest.TestCase):
    def test_candidate_surface_coalescing_state(self):
        source=(ROOT/'crates/rustui-gpui-fast/src/fast/change_present.rs').read_text()
        state=source[source.index('#[derive(Default)]'):source.index('#[derive(Default)]\nstruct State')]
        tests=source[source.index('#[cfg(test)]'):]
        with tempfile.TemporaryDirectory(prefix='rustui-g53b-cadence-') as directory:
            rust=Path(directory)/'state.rs';binary=Path(directory)/'state'
            switches=source[source.index('pub fn fast_coalesce_enabled'):source.index('#[derive(Default)]')]
            env=dict(os.environ)
            for key in ['GPUI_FAST_IMMEDIATE_COALESCE','GPUI_FAST_IMMEDIATE_ANY_CHANGE','GPUI_FAST_NATIVE_SCHEDULER','GPUI_FAST_TRACE_WAKES']:
                env.pop(key,None)
            scheduler = ROOT / 'crates/rustui-gpui-fast/src/fast/native_scheduler.rs'
            rust.write_text(
                '#![allow(dead_code)]\n'
                '#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,Hash)] pub struct WindowId(u64);\n'
                'fn fast_immediate_enabled()->bool{true}\n'
                f'#[path = "{scheduler}"] mod native_scheduler;\n'
                'mod change_present {\n'
                'use std::{time::{Instant,Duration},sync::OnceLock};\n'
                + switches + state + tests
                + '\n#[test]fn new_switches_default_off(){'
                  'assert!(!fast_coalesce_enabled());assert!(!fast_any_change_enabled());}\n}\n'
            )
            subprocess.run(['rustup','run','1.98.0','rustc','--edition=2024','--test',str(rust),'-o',str(binary)],cwd=ROOT,check=True)
            subprocess.run([str(binary)],cwd=ROOT,env=env,check=True)
    def test_cpu_admission_precedes_all_runtime_scene_builds(self):
        core=(ROOT/'crates/rustui-gpui-fast/src/fast/immediate_present.rs').read_text()
        body=core[core.index('pub(crate) fn try_draw('):core.index('pub(crate) fn after_input(')]
        self.assertLess(body.index('change_present::before_build'),body.index('window.draw(cx)'))
        window=(ROOT/'crates/rustui-gpui-fast/src/window.rs').read_text()
        frame=window[window.index('let should_draw = invalidator.is_dirty()'):window.index('pub fn capslock')]
        self.assertLess(frame.index('change_present::before_build'),frame.index('window.draw(cx)'))
        mac=(ROOT/'crates/rustui-gpui-fast-macos/src/fast/composition.rs').read_text()
        body=mac[mac.index('pub(crate) fn any_build_ready'):mac.index('pub(crate) fn any_submitted')]
        self.assertIn('ready |= surface.renderer.immediate_build_ready(window)',body)

    def test_draw_first_is_queued_after_local_edit_before_legacy_effects(self):
        source=(ROOT/'crates/rustui-gpui-base/src/input/base/state.rs').read_text()
        body=source[source.index('    fn emit_change('):source.index('    fn full_input_edit(')]
        self.assertLess(body.index('window.fast_draw_after_edit(cx)'),body.index('cx.emit(InputEvent::Change)'))
        core=(ROOT/'crates/rustui-gpui-fast/src/fast/edit_present.rs').read_text()
        self.assertIn('fast_defer_before_effects',core)
        self.assertNotIn('window.draw(',body) # never draw while editor entity is borrowed
        adapter=(ROOT/'crates/rustui-gpui-component/src/highlighter/input_adapter.rs').read_text()
        self.assertIn('if gpui::fast_draw_first_enabled() || text.len() > SYNC_PARSE_MAX_BYTES',adapter)
        self.assertIn('!highlighter.borrow().text().eq(&text_for_apply)',adapter)
if __name__=='__main__':unittest.main()
