from pathlib import Path
import sys,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from g5_slots import diagnose

class SlotDiagnosisTests(unittest.TestCase):
    def test_trace_distinguishes_backpressure_gpu_work_and_late_or_superseded_frames(self):
        result=diagnose([1,10,20,30,40,50,60,70],[(5,1),(45,2),(48,3)],
                        [(1,15,25,35),(2,46,47,None),(3,49,49,60)],[6],0)
        self.assertEqual([r['reason'] for r in result['rows']],
                         ['no new input','no drawable free','frame not ready','WindowServer late',None,'presented but replaced',None,'no new input'])
        self.assertEqual(result['missing_new_version_slots'],6)
    def test_control_floor_prevents_calling_fixed_pipeline_delay_a_compositor_miss(self):
        result=diagnose([10,20,30,40],[(5,1)],[(1,6,9,40)],[],2)
        self.assertEqual(result['rows'][1]['reason'],'frame not ready')
        self.assertEqual(result['rows'][2]['reason'],'WindowServer late')

if __name__=='__main__':unittest.main()
