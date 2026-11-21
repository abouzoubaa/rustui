"""Exercise the opt-in recorder extracted from the applied candidate."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class FrameRecorderTests(unittest.TestCase):
    def test_disabled_is_silent_and_enabled_records_draws_and_present_gaps(self):
        source_path = ROOT / "vendor/gpui-fast-0.1.14/src/window.rs"
        if not source_path.exists():
            source_path = ROOT / "crates/rustui-gpui/src/window.rs"
        source = source_path.read_text()
        start = source.index("// Opt-in frame recording for native acceptance;")
        end = source.index("const FRAME_INTERVAL_TOLERANCE:", start)
        recorder = (ROOT / "vendor/gpui-fast-0.1.14/src/fast/frame_recorder.rs").read_text()
        # Keep the real implementation, with only GPUI's WindowId replaced.
        recorder = "\n".join(line for line in recorder.splitlines() if not line.startswith("//!"))
        harness = "type WindowId = u64;\nmod fast { pub mod frame_recorder {\n" + recorder + "\n} }\n" + source[start:end] + '''
fn main() {
    f0_frame_record(1, "layout", f0_phase_start());
    f0_frame_record(1, "draw", f0_frame_start());
    f0_frame_record(1, "present", f0_frame_start());
    f0_frame_record(1, "present", f0_frame_start());
    fast::frame_recorder::after_present();
}
'''
        channel = next(line.split('"')[1] for line in (ROOT / "rust-toolchain.toml").read_text().splitlines() if line.strip().startswith("channel"))
        with tempfile.TemporaryDirectory() as temporary:
            rust = Path(temporary) / "recorder.rs"
            binary = Path(temporary) / "recorder"
            rust.write_text(harness)
            subprocess.run(["rustup", "run", channel, "rustc", "--edition", "2024", str(rust), "-o", str(binary)], check=True, capture_output=True)
            env = dict(os.environ, RUSTUI_FRAME_LOG="0")
            disabled = subprocess.run([str(binary)], env=env, check=True, capture_output=True, text=True)
            self.assertEqual(disabled.stderr, "")
            env["RUSTUI_FRAME_LOG"] = "1"
            env["RUSTUI_FRAME_PROFILE"] = "0"
            enabled = subprocess.run([str(binary)], env=env, check=True, capture_output=True, text=True)
            frames = [json.loads(line.removeprefix("RUSTUI_FRAME_LOG ")) for line in enabled.stderr.splitlines()]
            self.assertEqual([frame["event"] for frame in frames], ["draw", "present", "present"])
            self.assertIsNone(frames[1]["gap_ms"])
            self.assertGreaterEqual(frames[2]["gap_ms"], 0)
            self.assertTrue(all(frame["cpu_ms"] >= 0 for frame in frames))
            self.assertTrue(all(frame["epoch_ms"] > 0 for frame in frames))
            env["RUSTUI_FRAME_PROFILE"] = "1"
            profiled = subprocess.run([str(binary)], env=env, check=True, capture_output=True, text=True)
            self.assertEqual(json.loads(profiled.stderr.splitlines()[0].removeprefix("RUSTUI_FRAME_LOG "))["event"], "layout")

    def test_buffered_record_bytes_match_original_and_flush_on_process_exit(self):
        recorder = (ROOT / "vendor/gpui-fast-0.1.14/src/fast/frame_recorder.rs").read_text()
        recorder = "\n".join(line for line in recorder.splitlines() if not line.startswith("//!"))
        harness = "type WindowId = u64;\n" + recorder.replace("use crate::WindowId;", "") + r'''
fn main() {
    for gap in [None, Some(0.0), Some(8.333333333)] {
        let record = Record { window: 42, event: "layout", time_ms: 12.3456789,
            cpu_ms: 0.000123456, gap_ms: gap, epoch_ms: 1795267351234.5678 };
        let mut buffered = String::new();
        record.append(&mut buffered);
        let original = format!(
            "RUSTUI_FRAME_LOG {{\"event\":\"{}\",\"window\":\"{:?}\",\"time_ms\":{:.6},\"cpu_ms\":{:.6},\"gap_ms\":{},\"epoch_ms\":{:.6}}}\n",
            record.event, record.window, record.time_ms, record.cpu_ms,
            gap.map_or_else(|| "null".into(), |gap| format!("{gap:.6}")), record.epoch_ms);
        assert_eq!(buffered.as_bytes(), original.as_bytes());
    }
    record(1, "layout", phase_start());
    record(1, "draw", start());
    record(1, "present", start());
    assert!(!RECORDS.lock().unwrap().is_empty());
    // process::exit skips Rust destructors; the C exit handler must still drain.
    std::process::exit(0);
}
'''
        channel = next(line.split('"')[1] for line in (ROOT / "rust-toolchain.toml").read_text().splitlines() if line.strip().startswith("channel"))
        with tempfile.TemporaryDirectory() as temporary:
            rust, binary = Path(temporary)/"bytes.rs", Path(temporary)/"bytes"
            rust.write_text(harness)
            result = subprocess.run(["rustup", "run", channel, "rustc", "--edition", "2024", str(rust), "-o", str(binary)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(binary)], env=dict(os.environ, RUSTUI_FRAME_LOG="buffered", RUSTUI_FRAME_PROFILE="1"), capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            frames = [json.loads(line.removeprefix("RUSTUI_FRAME_LOG ")) for line in result.stderr.splitlines()]
            self.assertEqual([frame["event"] for frame in frames], ["layout", "draw", "present"])

    def test_bounce_oracle_is_opt_in_and_keeps_visual_and_logical_offsets_separate(self):
        source = (ROOT / "crates/rustui-gpui-base/src/scroll_bounce.rs").read_text()
        start = source.index("// Opt-in observation of boundary displacement;")
        end = source.index("/// Motion tokens", start)
        channel = next(line.split('"')[1] for line in (ROOT / "rust-toolchain.toml").read_text().splitlines() if line.strip().startswith("channel"))
        with tempfile.TemporaryDirectory() as temporary:
            rust, binary = Path(temporary)/"oracle.rs", Path(temporary)/"oracle"
            rust.write_text(source[start:end] + 'fn main() { f0_grid_visual_oracle(false,|| panic!("non-grid read"),-20.); f0_grid_visual_oracle(true,|| { assert_eq!(std::env::var("RUSTUI_F0_PIXELS").unwrap(),"1"); -1000. },-20.); }')
            build = subprocess.run(["rustup","run",channel,"rustc",str(rust),"-o",str(binary)],capture_output=True,text=True)
            self.assertEqual(build.returncode,0,build.stderr)
            env = dict(os.environ,RUSTUI_F0_PIXELS="0",RUSTUI_F0_PANEL="grid")
            self.assertEqual(subprocess.run([str(binary)],env=env,capture_output=True,text=True,check=True).stderr, "")
            env["RUSTUI_F0_PIXELS"]="1"
            result = subprocess.run([str(binary)],env=env,capture_output=True,text=True,check=True)
            lines = result.stderr.splitlines()
            self.assertEqual(len(lines),1)
            entry = json.loads(lines[0].removeprefix("RUSTUI_VISUAL_ORACLE_LOG "))
            self.assertEqual(entry["source"]["offset_y"],-1000)
            self.assertEqual(entry["source"]["visual_offset_y"],-20)
            for panel in ("grid", "list", "keyed"):
                env["RUSTUI_F0_PANEL"] = panel
                result = subprocess.run([str(binary)],env=env,capture_output=True,text=True,check=True)
                entry = json.loads(result.stderr.removeprefix("RUSTUI_VISUAL_ORACLE_LOG "))
                self.assertEqual(entry["source"]["panel"], panel)
                self.assertEqual(entry["source"]["visual_offset_y"], -20)


if __name__ == "__main__":
    unittest.main()
