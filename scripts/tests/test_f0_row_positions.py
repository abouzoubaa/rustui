import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("row_positions", ROOT / "scripts/f0_row_positions.py")
positions = importlib.util.module_from_spec(spec)
spec.loader.exec_module(positions)


class RowPositionTests(unittest.TestCase):
    def test_capture_delay_is_calibrated_but_wrong_rows_are_still_detected(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            oracles, frames = [], []
            for index in range(16):
                row = 0 if index == 0 else 40 + index * 30
                timestamp = 1000 + index * 10
                oracles.append(dict(epoch_ms=timestamp - 30, source=dict(offset_y=-row * 30)))
                frames.append(dict(frame=index, epoch_ms=timestamp, pitch=60, texts=[dict(text=f"F0 row {row}", center=30)]))
            frames[-1]["texts"][0]["text"] = "F0 row 9999"
            (folder / "calibration.json").write_text(json.dumps(dict(text_shift_px=30, pitch=60, scale=2)))
            (folder / "setup.json").write_text(json.dumps(dict(viewport=dict(scale=2, y=0), oracles=oracles)))
            (folder / "rows.jsonl").write_text("\n".join(map(json.dumps, frames)))
            result = positions.analyze(folder, "list")
            self.assertEqual(result["capture_delay_ms"], 30)
            self.assertEqual(result["stale_rows"], 1)
            self.assertEqual(result["frames_without_aligned_source"], 0)
            self.assertEqual(len((folder / "row-positions.jsonl").read_text().splitlines()), 16)

    def test_capture_starting_already_scrolled_uses_recorded_warmup_origin(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            frames, oracles = [], []
            for index in range(16):
                row = 40 + index * 30
                timestamp = 1000 + index * 10
                frames.append(dict(frame=index, epoch_ms=timestamp, pitch=60, texts=[dict(text=f"F0 row {row}", center=30)]))
                oracles.append(dict(epoch_ms=timestamp, source=dict(offset_y=-row * 30)))
            (folder / "setup.json").write_text(json.dumps(dict(viewport=dict(scale=2, y=0), oracles=oracles)))
            (folder / "calibration.json").write_text(json.dumps(dict(text_shift_px=30, pitch=60, scale=2)))
            (folder / "rows.jsonl").write_text("\n".join(map(json.dumps, frames)))
            result = positions.analyze(folder, "list")
            self.assertEqual(result["stale_rows"], 0)
            self.assertEqual(result["capture_delay_ms"], 0)

    def test_partial_boundary_glyphs_are_excluded_but_full_wrong_labels_remain(self):
        frame = dict(height=1037, pitch=60, texts=[
            dict(text="F0 row 1637", center=10),
            dict(text="F0 row 1838", center=68),
            dict(text="F0 row 9999", center=128),
            dict(text="F0 row 3854", center=1028)])
        self.assertEqual(positions.anchors(frame,"collection"), [(1838,68),(9999,128)])

    def test_boundary_bounce_is_a_visual_displacement_not_a_stale_row(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            (folder / "calibration.json").write_text(json.dumps(dict(text_shift_px=100,pitch=64,scale=2)))
            (folder / "setup.json").write_text(json.dumps(dict(viewport=dict(scale=2,y=0),oracles=[dict(epoch_ms=1000,source=dict(offset_y=-31575.5,visual_offset_y=-20))])))
            (folder / "rows.jsonl").write_text(json.dumps(dict(frame=1,epoch_ms=1000,pitch=64,texts=[dict(text="Product 988",center=100+987*64-31575.5*2-40),dict(text="Product 9999",center=200)])))
            result = positions.analyze(folder,"grid")
            self.assertEqual(result["stale_rows"],1)
            self.assertEqual(result["recognized_row_observations"],2)

    def test_editor_ignores_body_numbers_and_clipped_glyphs_but_keeps_wrong_gutter_IDs(self):
        frame=dict(height=1037,pitch=40,texts=[dict(text="5312",center=13,x=28),dict(text="1",center=500,x=422),dict(text="9999",center=540,x=28)])
        self.assertEqual(positions.anchors(frame,"editor"),[(9999,540)])

    def test_coarse_source_cadence_keeps_strict_counts_and_rejects_wrong_or_held_rows(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            oracles = [dict(epoch_ms=1000+i*33, source=dict(offset_y=-(40+i*30)*30)) for i in range(30)]
            frames = [dict(frame=i,epoch_ms=1010+i*33,pitch=60,texts=[dict(text=f"F0 row {40+i*30}",center=30)]) for i in range(19)]
            frames[16]["epoch_ms"] += 25  # within one observer interval
            frames[17]["texts"][0]["text"] = "F0 row 9999"
            frames[18]["epoch_ms"] += 70  # genuinely older than one interval
            (folder/"setup.json").write_text(json.dumps(dict(viewport=dict(scale=2,y=0),oracles=oracles)))
            (folder/"calibration.json").write_text(json.dumps(dict(text_shift_px=30,pitch=60,scale=2)))
            (folder/"rows.jsonl").write_text("\n".join(map(json.dumps,frames)))
            result = positions.analyze(folder,"list")
            self.assertEqual(result["stale_rows"],3)
            self.assertEqual(result["cadence_aware_position_candidates"],2)
            self.assertEqual(result["alignment_tolerance_ms"],34)
            self.assertEqual(result["cadence_frames_without_source"],0)
