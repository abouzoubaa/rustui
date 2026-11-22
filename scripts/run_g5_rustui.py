#!/usr/bin/env python3
"""Frozen-build G5-3 pairs; hover/drag triplets share their middle capture."""
import argparse
from datetime import datetime,timezone
import hashlib,json,os,subprocess,time
from pathlib import Path
import g5_latency as metrics
import g5_slots

ROOT=Path(__file__).resolve().parents[1]
GROUPS=[('hover','hover','0'),('drag','drag','0'),('typing-hl-on','typing','1'),
        ('typing-hl-off','typing','0'),('log','log','0'),('acceptance','acceptance','0')]
ARMS={'off':('0','0'),'immediate':('1','0'),'reserve':('1','1')}


def capture(out,group,scene,hl,pair,arm,seconds):
    binary=Path.home()/'.cache/rustui-target/release/rustui-lab'
    stem=f'{group}-{pair}-{arm}';log=out/(stem+'.log')
    if log.exists():raise RuntimeError(f'Refusing to overwrite {log}')
    flags=dict(os.environ,RUSTUI_G5_REFERENCE='1',RUSTUI_DAMAGE='0',RUSTUI_FRAME_LOG='buffered',
               RUSTUI_FRAME_PROFILE='1',RUSTUI_G0_LATENCY='1',RUSTUI_LATENCY_BREAKDOWN='1',RUSTUI_G5_REFRESH='1',
               RUSTUI_HL_FASTPATH=hl,GPUI_FAST_IMMEDIATE_PRESENT=ARMS[arm][0],
               GPUI_FAST_DRAWABLE_RESERVE_TWO=ARMS[arm][1],GPUI_FAST_VIEW_DISPLAY_LINK='0',GPUI_FAST_FORCE_LEGACY_LINK='0',
               GPUI_FAST_FRAME_AHEAD='0',GPUI_FAST_IMMEDIATE_HOLD_MS='250',GPUI_FAST_IMMEDIATE_TRANSACTION='0',
               RUSTUI_F0_PANEL='collection',DYLD_INSERT_LIBRARIES=str(ROOT/'target/headstart/g5-1/libG5Refresh.dylib'))
    for key in ['RUSTUI_G0_HOVER_PIXELS','RUSTUI_G5_PIXEL_CAPTURE','MTL_HUD_ENABLED','RUSTUI_F0_SCROLL','RUSTUI_G5_OVERLAY']:
        flags.pop(key,None)
    print(stem,'start',datetime.now(timezone.utc).isoformat(),flush=True)
    before=metrics.collect_load(str(out/(stem+'.top-start.txt')),[str(binary)],None)
    (out/(stem+'.top-five.json')).write_text(json.dumps(before['top_processes'],indent=2)+'\n')
    with log.open('w') as stream:
        subprocess.run([str(binary),'--g0-scene',scene,'--duration',str(seconds)],cwd=ROOT,env=flags,stdout=stream,stderr=subprocess.STDOUT,check=True,timeout=seconds+45)
    after=metrics.collect_load(str(out/(stem+'.top-end.txt')),[str(binary)],None)
    records=metrics.records(log);x=metrics.bare(records,2)
    metrics.annotate_load(x,before,after)
    assert x['final_text_valid'] and x['out_of_order_presentations']==0,(stem,'correctness',x['out_of_order_presentations'])
    assert x['last_version_shown']==x['expected_final_version'],(stem,'last version',x['last_version_shown'],x['expected_final_version'])
    events=records['RUSTUI_PX4_LOG '];latest={};violations=[];counts=[]
    for event in events:
        if event.get('kind')=='g5_acquire_start':latest[event['window']]=event
        if event.get('kind')=='next_drawable':
            pre=latest.get(event['window']);assert pre is not None
            count,pool,_,attached=pre['values'];counts.append(count)
            if arm!='off' and not(attached==0 and (count<=pool-2 if arm=='reserve' else count<pool)):
                violations.append(pre)
    assert not violations,(stem,violations)
    x.update(group=group,scene=scene,hl=hl,pair=pair,arm=arm,capacity_violations=len(violations),max_pre_acquire_in_flight=max(counts,default=0))
    if scene in ('hover','drag'):
        slots=g5_slots.from_records(records,2)
        (out/(stem+'.slots.json')).write_text(json.dumps(slots,indent=2)+'\n')
        x['missed_slots']={k:v for k,v in slots.items() if k!='rows'}
    (out/(stem+'.json')).write_text(json.dumps(x,indent=2)+'\n')
    print(stem,'complete',f"latency={x['input_to_display_ms']['median']:.3f}/{x['input_to_display_ms']['p95']:.3f}",
          f"shown/s={x['distinct_versions_per_second']:.3f}",f"CPU={x['cpu_s_per_attempt']*1000:.3f}",
          f">2ms={x['next_drawable_over_2_ms']}",flush=True)
    return x


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--deadline',required=True)
    parser.add_argument('--functional',action='store_true')
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    paths=[Path.home()/'.cache/rustui-target/release/rustui-lab',ROOT/'target/headstart/g5-1/libG5Refresh.dylib']
    frozen={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    (args.output/'frozen-build.json').write_text(json.dumps(frozen,indent=2)+'\n')
    deadline=datetime.fromisoformat(args.deadline.replace('Z','+00:00')).timestamp();rows=[]
    try:
        for group,scene,hl in GROUPS:
            for pair in range(1,2 if args.functional else 6):
                needed=35 if args.functional else 215 if scene in ('hover','drag') else 145
                if time.time()+needed>deadline:
                    print('Timebox: no new matched triplet started.',flush=True);return 75
                arms=['reserve'] if args.functional else (['off','immediate','reserve'] if pair%2 else ['reserve','immediate','off']) if scene in ('hover','drag') else (['off','immediate'] if pair%2 else ['immediate','off'])
                for arm in arms:
                    rows.append(capture(args.output,group,scene,hl,pair,arm,5 if args.functional else 60))
                    (args.output/'captures.json').write_text(json.dumps(rows,indent=2)+'\n')
        return 0
    finally:
        current={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
        (args.output/'frozen-build-after.json').write_text(json.dumps(current,indent=2)+'\n')
        assert current==frozen,'executables changed during timing'

if __name__=='__main__':raise SystemExit(main())
