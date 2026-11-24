import sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from g5_budget import decompose
class BudgetTests(unittest.TestCase):
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
if __name__=='__main__':unittest.main()
