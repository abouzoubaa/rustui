"""Fixed CSV coverage for baseline parsing, medians, spread and JSON."""
import importlib.util
import json
import tempfile
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("perf_baseline", ROOT / "scripts/perf_baseline.py")
PERF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PERF)

SAMPLE = """test workload ... rows,mode,sample,updates,update_renders,draw_renders,elapsed_ns,ns_per_update
10,update,1,10,10,0,10000000,1000000
10,update,2,10,10,0,30000000,3000000
10,update,3,10,10,0,20000000,2000000
10,forced_draw,1,10,0,10,10000000,1000000
10,forced_draw,2,10,0,10,30000000,3000000
10,forced_draw,3,10,0,10,20000000,2000000
100,update,1,10,10,0,10000000,1000000
100,update,2,10,10,0,30000000,3000000
100,update,3,10,10,0,20000000,2000000
100,forced_draw,1,10,0,10,10000000,1000000
100,forced_draw,2,10,0,10,30000000,3000000
100,forced_draw,3,10,0,10,20000000,2000000
1000,update,1,10,10,0,10000000,1000000
1000,update,2,10,10,0,30000000,3000000
1000,update,3,10,10,0,20000000,2000000
1000,forced_draw,1,10,0,10,100000000,10000000
1000,forced_draw,2,10,0,10,120000000,12000000
1000,forced_draw,3,10,0,10,110000000,11000000
ok
"""


class PerformanceBaselineTests(unittest.TestCase):
    def test_parses_fixed_csv_and_reports_median_and_json(self):
        run = PERF.parse_csv(SAMPLE, expected_samples=3)
        data = PERF.build_baseline([run, run, run], {"commit": "fixture"})
        self.assertEqual(data["median_ms_per_update"]["10"]["update"], 2.0)
        self.assertEqual(data["median_ms_per_update"]["1000"]["forced_draw"], 11.0)
        self.assertEqual(data["forced_draw_1000_spread_percent"], 0.0)
        self.assertTrue(data["reproducible"])
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "perf_baseline.json"
            PERF.write_baseline(path, data)
            self.assertEqual(json.loads(path.read_text()), data)
            self.assertEqual(json.loads(path.read_text())["reference_ms"], 12.2)
        self.assertIn("1,000 rows, update + forced draw | 11.0000 | -9.8% | MISS", PERF.format_table(data))

    def test_spread_flags_over_fifteen_percent(self):
        first = PERF.parse_csv(SAMPLE, expected_samples=3)
        high = {key: list(values) for key, values in first.items()}
        high[(1000, "forced_draw")] = [20.0, 21.0, 22.0]
        data = PERF.build_baseline([first, first, high], {})
        self.assertFalse(data["reproducible"])
        self.assertAlmostEqual(data["forced_draw_1000_spread_percent"], 1000 / 11)

    def test_rejects_missing_or_duplicate_samples(self):
        with self.assertRaisesRegex(ValueError, "missing workload samples"):
            PERF.parse_csv(SAMPLE.replace("10,update,1,10,10,0,10000000,1000000\n", ""), expected_samples=3)
        with self.assertRaisesRegex(ValueError, "duplicate"):
            PERF.parse_csv(SAMPLE + "10,update,1,10,10,0,10000000,1000000\n", expected_samples=3)


if __name__ == "__main__":
    unittest.main()
