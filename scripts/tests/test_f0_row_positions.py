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
            (folder / "setup.json").write_text(json.dumps(dict(viewport=dict(scale=2, y=0), oracles=oracles)))
            (folder / "rows.jsonl").write_text("\n".join(map(json.dumps, frames)))
            result = positions.analyze(folder, "list")
            self.assertEqual(result["capture_delay_ms"], 30)
            self.assertEqual(result["stale_rows"], 1)
            self.assertEqual(result["frames_without_aligned_source"], 0)
            self.assertEqual(len((folder / "row-positions.jsonl").read_text().splitlines()), 16)
