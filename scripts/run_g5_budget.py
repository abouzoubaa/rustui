#!/usr/bin/env python3
"""Native-event G5-L1 capture. Build beforehand; this runner never compiles."""
import argparse,json,os,subprocess
from pathlib import Path
import g5_budget,g5_latency
ROOT=Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--app',choices=['bare','lab'],required=True)
    p.add_argument('--scene',choices=['typing','hover','drag'],default='typing')
    p.add_argument('--seconds',type=int,default=60)
    p.add_argument('--hz',choices=['10','30'],default='10')
    p.add_argument('--timeout',type=float)
    p.add_argument('--scheduler',choices=['0','1'],default='0')
    p.add_argument('--trace-wakes',action='store_true')
    p.add_argument('--switches',help='Four bits: immediate, draw-first, coalescing, any-change')
    p.add_argument('--passive',choices=['0','1'],required=True)
    p.add_argument('--pixels',choices=['0','1'],required=True)
    p.add_argument('--immediate',choices=['0','1'],default='1')
    p.add_argument('--any-change',choices=['0','1'],default='1')
    p.add_argument('--sparse',action='store_true')
    p.add_argument('--native-helper',type=Path)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
    assert 1<=a.seconds<=3600
    refs=ROOT/'target/headstart/g5-1'
    helper=a.native_helper or refs/'libG5Native.dylib'
    libs=[helper]+([refs/'libG5Pixels.dylib',refs/'libG5Refresh.dylib'] if a.app=='lab' else [])
    binary=refs/'G5 Bare.app/Contents/MacOS/latency-gpui' if a.app=='bare' else Path.home()/'.cache/rustui-target/release/rustui-lab'
    arm=a.switches or f'{a.immediate}{a.immediate}{a.immediate}{a.any_change}'
    if len(arm)!=4 or set(arm)-set('01'):
        p.error('--switches expects four bits')
    flags=dict(os.environ,RUSTUI_L1_NATIVE='1',RUSTUI_L1_LAB=str(int(a.app=='lab')),RUSTUI_L1_SPARSE=str(int(a.sparse)),
        RUSTUI_G5_REFERENCE='1',RUSTUI_DAMAGE='0',RUSTUI_FRAME_LOG='buffered',RUSTUI_FRAME_PROFILE='1',RUSTUI_G0_LATENCY='1',RUSTUI_LATENCY_BREAKDOWN='1',
        RUSTUI_G5_PIXEL_CAPTURE=a.pixels,RUSTUI_G5_REFRESH=a.passive,RUSTUI_G5_SCENE=a.scene,RUSTUI_G5_INPUT_HZ=a.hz,RUSTUI_G0_TYPING_HZ=a.hz,
        RUSTUI_HL_FASTPATH='1' if a.app=='lab' and a.scene=='typing' else '0',GPUI_FAST_IMMEDIATE_PRESENT=arm[0],
        RUSTUI_DRAW_FIRST=arm[1],GPUI_FAST_IMMEDIATE_COALESCE=arm[2],GPUI_FAST_IMMEDIATE_ANY_CHANGE=arm[3],
        GPUI_FAST_NATIVE_SCHEDULER=a.scheduler,GPUI_FAST_TRACE_WAKES=str(int(a.trace_wakes)),
        GPUI_FAST_IMMEDIATE_HOLD_MS='250',GPUI_FAST_VIEW_DISPLAY_LINK='0',GPUI_FAST_FORCE_LEGACY_LINK='0',GPUI_FAST_FRAME_AHEAD='0',
        GPUI_FAST_IMMEDIATE_TRANSACTION='0',DYLD_INSERT_LIBRARIES=':'.join(str(f.resolve()) for f in libs))
    assert not a.sparse or (a.app=='bare' and a.seconds==120 and a.scene=='typing')
    before=g5_latency.collect_load(str(a.output/'load-start.txt'),[str(binary)],None)
    import hashlib
    frozen={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in [binary,*libs]}
    (a.output/'method.json').write_text(json.dumps(dict(args=vars(a),flags={k:v for k,v in flags.items() if k.startswith(('RUSTUI_','GPUI_FAST_','DYLD_'))},frozen=frozen,load=before),indent=2,default=str)+'\n')
    cmd=[str(binary),str(a.seconds)] if a.app=='bare' else [str(binary),'--g0-scene',a.scene,'--duration',str(a.seconds)]
    log=a.output/'capture.log'
    try:
        with log.open('w') as out:result=subprocess.run(cmd,cwd=ROOT,env=flags,stdout=out,stderr=subprocess.STDOUT,timeout=a.timeout or a.seconds+20)
        assert result.returncode==0,('app exit',result.returncode)
    except subprocess.TimeoutExpired:
        (a.output/'failure.json').write_text(json.dumps({'error':'workload did not finish','timeout_s':a.timeout or a.seconds+20})+'\n')
        raise
    assert frozen=={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in [binary,*libs]},'build changed during capture'
    after=g5_latency.collect_load(str(a.output/'load-end.txt'),[str(binary)],None)
    x=g5_budget.analyse(log);g5_latency.annotate_load(x,before,after)
    load=x['machine_load']
    if max(load['start_background_cpu_percent'],load['end_background_cpu_percent']) <= 10:
        load['label']=(f"quiet capture ({load['start_background_cpu_percent']:.2f}% background at start; "
                       f"{load['end_background_cpu_percent']:.2f}% after)")
    load.pop('owner_authorized_bypass',None)
    (a.output/'summary.json').write_text(json.dumps(x,indent=2)+'\n')
    print(json.dumps({k:v for k,v in x.items() if k not in ('rows','next_drawable_outliers')},indent=2))
if __name__=='__main__':main()
