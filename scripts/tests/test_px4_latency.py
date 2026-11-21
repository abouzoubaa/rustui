import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from px4_latency import summarize

class PX4Tests(unittest.TestCase):
    def test_real_mach_clock_scopes_and_unused_drawable_accounting(self):
        source=(ROOT/'vendor/gpui-fast-0.1.14/src/fast/latency_breakdown.rs').read_text().split('fn flush()')[0]
        harness='type WindowId=u64;\nmod trace {\n'+source+'''
fn flush() {}
pub fn exercise() {
 if !fast_latency_enabled() {handled(7,"typing",1,1);draw(7);assert!(RECORDS.lock().unwrap().is_none());return}
 let before=fast_latency_tick();
 let callback=fast_latency_callback(7,before);
 handled(7,"typing",1,1); requested(7); draw(7);painted(7,"typing",1,1);
 let first=fast_latency_acquired(7,99).unwrap();
 assert!(fast_latency_acquired(7,99).is_none());
 {let _scope=present_scope(7);let frame=fast_latency_frame().unwrap();assert_eq!(frame.window(),7);let token=fast_latency_command(frame,99).unwrap();fast_latency_committed(Some(token));fast_latency_completed(token,1.);}
 assert!(fast_latency_frame().is_none());
 fast_latency_displayed(first,1.,1.001);
 let second=fast_latency_acquired(7,99).unwrap();assert!(second.id>first.id);
 fast_latency_released(7,99);
 // A late callback for a released acquisition cannot retire a newer one.
 let third=fast_latency_acquired(7,99).unwrap();fast_latency_displayed(second,0.,1.);
 let mut g=RECORDS.lock().unwrap();let r=g.as_mut().unwrap();assert_eq!(r.windows[&7].live[&99],third.id);
 assert!(r.records.iter().filter_map(|r|if let Rec::Acquire(_,n,_)=r{Some(*n)}else{None}).all(|n|n==1));
 assert!(fast_latency_tick()>=before);drop(g);drop(callback);
 assert!(CALLBACK.with(Cell::get).is_none());
}
}
fn main(){trace::exercise()}
'''
        with tempfile.TemporaryDirectory() as tmp:
            file,binary=Path(tmp)/'mach.rs',Path(tmp)/'mach'
            file.write_text(harness)
            built=subprocess.run(['rustup','run','1.98.0','rustc','--edition=2024',str(file),'-o',str(binary)],capture_output=True,text=True)
            self.assertEqual(built.returncode,0,built.stderr)
            for enabled in ('0','1'):
                run=subprocess.run([str(binary)],env=dict(os.environ,RUSTUI_LATENCY_BREAKDOWN=enabled),capture_output=True,text=True)
                self.assertEqual(run.returncode,0,run.stderr)

    def test_stage_chain_uses_first_callback_and_preserves_signed_late_completion(self):
        events=[dict(event='clock',numer=1000000,denom=1,mach_absolute_time=True)]
        def e(event,**kw): events.append(dict(event=event,window='w',**kw))
        e('handled',scene='typing',first=1,version=1,mach=10)
        e('requested',mach=11);e('callback',mach=13,callback=1)
        e('callback',mach=16,callback=2);e('draw',draw=1,callback=2,mach=17)
        e('painted',scene='typing',first=1,version=1,draw=1,mach=18)
        e('attempt',draw=1,attempt=1,callback=2,mach=19)
        e('committed',draw=1,attempt=1,drawable=1,mach=20,gpu_end_host=0)
        e('gpu_completed',draw=1,attempt=1,drawable=1,mach=25,gpu_end_host=100.021)
        e('displayed',drawable=1,callback_mach=30,presented_host=100.024,callback_host=100.030)
        e('config',kind='metal_layer',values=[3,1,1,0],mach=1)
        e('config',kind='metal_layer',values=[3,1,0,0],mach=8)
        e('config',kind='metal_layer',values=[3,1,1,0],mach=31)
        e('acquired',drawable=1,key=99,in_flight=1,mach=10)
        e('acquired',drawable=2,key=100,in_flight=3,mach=18)
        old=dict(event='input',window='w',scene='typing',version=1,epoch_ms=1000)
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'frames.log';lines=['RUSTUI_PX4_LOG '+json.dumps(r) for r in events]
            lines+=['RUSTUI_INPUT_LOG '+json.dumps(old),'RUSTUI_G0_LOG '+json.dumps(dict(event='start',scene='typing',epoch_ms=999,mach=9)), 'RUSTUI_G0_LOG '+json.dumps(dict(event='end',valid=True,scene='typing',epoch_ms=1001,mach=11))]
            p.write_text('\n'.join(lines))
            r=summarize(p)
            for row in events:
                if row['event'] in ('draw','attempt'): row['callback']=0
            lines=['RUSTUI_PX4_LOG '+json.dumps(row) for row in events]+lines[len(events):]
            p.write_text('\n'.join(lines))
            outside=summarize(p)
        self.assertEqual(r['coverage']['complete_chain'],1)
        self.assertEqual(outside['coverage'].get('complete_chain',0),0)
        self.assertEqual(outside['stages']['requested_to_next_callback']['samples'],0)
        self.assertEqual(outside['stages']['callback_to_draw']['samples'],0)
        values=[r['stages'][key]['median_ms'] for key in ('handled_to_requested','requested_to_next_callback','callback_to_draw','draw_to_committed','committed_to_gpu_completed','gpu_completed_to_display')]
        for got,want in zip(values,[1,2,4,3,5,-1]): self.assertAlmostEqual(got,want)
        self.assertAlmostEqual(r['input_to_screen']['median_ms'],14)
        self.assertEqual(r['completion_callback_after_display'],1)
        self.assertEqual(r['settings']['w']['metal_layer'],[3,1,0,0])
        self.assertEqual(r['in_flight']['w']['maximum'],1)

if __name__=='__main__':unittest.main()
