import json,subprocess,sys,tempfile,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from g5_budget import analyse_appkit,check_damage_allocations,decompose
class BudgetTests(unittest.TestCase):
    def test_allocation_failure_rejects_capture_even_after_recovery(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "capture.log"
            success = 'RUSTUI_FRAME_LOG {"event":"damage_verify","mode":"partial"}\n'
            path.write_text(success)
            check_damage_allocations(path)
            for marker in ("RUSTUI_DAMAGE_WARNING", "RUSTUI_DAMAGE_CAPTURE_FAILED"):
                path.write_text(f"{marker} alloc_failed requested_bytes=16384 limit_bytes=18023\n" + success)
                with self.assertRaisesRegex(RuntimeError, "alloc_failed"):
                    check_damage_allocations(path)

    def test_scroll_is_rejected_for_the_bare_reference_before_creating_output(self):
        runner = Path(__file__).resolve().parents[1] / 'run_g5_budget.py'
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'capture'
            result = subprocess.run(
                [sys.executable, str(runner), '--app', 'bare', '--scene', 'scroll',
                 '--passive', '0', '--pixels', '1', '--output', str(output)],
                capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn('the scroll scene requires --app lab', result.stderr)
            self.assertFalse(output.exists())

    def test_formula_both_clocks_and_signed_s(self):
        r=decompose(10,10.002,10.011,10.010,.004)
        self.assertAlmostEqual(r['P'],2);self.assertAlmostEqual(r['wait'],5)
        self.assertAlmostEqual(r['S'],-1)
        self.assertAlmostEqual(r['sck_residual'],0)
        self.assertAlmostEqual(r['presented_residual'],0)
    def test_cannot_sum_stage_medians(self):
        import statistics as s
        rows=[decompose(0,p,p+w,p+w+x,0) for p,w,x in [(1,9,0),(4,4,0),(9,1,0)]]
        self.assertNotEqual(sum(s.median(r[k] for r in rows) for k in ['P','wait','S']),s.median(r['presented_total'] for r in rows))
    def test_appkit_uses_observer_clock_and_has_no_metal_clock(self):
        events = [
            ('G5_REF', dict(event='start', numer=1, denom=1)),
            ('G5_REF', dict(event='handled', version=1, mach=1_000_010_000)),
            ('G5_REF', dict(event='draw', version=1, mach=1_000_020_000)),
            ('G5_REF', dict(event='end', valid=True, inputs=1)),
            ('L1_NATIVE', dict(event='native', version=1, consumed=True,
                               hardware_s=.999, handler_mach=1_000_000_000)),
            ('G5_PIXELS', dict(event='displayed_pixels', version=1, display_mach=1_010_000_000)),
        ]
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'native.log'
            log.write_text(''.join(f'{prefix} {json.dumps(event)}\n' for prefix,event in events))
            result = analyse_appkit(log)
        self.assertFalse(result['presented_time_available'])
        self.assertAlmostEqual(result['stats']['sck_total']['median'], 10)
        self.assertAlmostEqual(result['stats']['hardware_sck_total']['median'], 11)
if __name__=='__main__':unittest.main()
