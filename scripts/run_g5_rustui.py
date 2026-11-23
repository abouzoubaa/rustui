#!/usr/bin/env python3
"""Frozen-build G5-3b matched pairs; CPU verdict uses total process CPU/second."""
import argparse
from datetime import datetime,timezone
import hashlib,json,os,subprocess,time
from pathlib import Path
import g5_latency as metrics
import g5_slots

ROOT=Path(__file__).resolve().parents[1]
GROUPS=[('typing-10','typing','1'),('typing-30','typing','1'),('hover','hover','0'),
        ('drag','drag','0'),('acceptance','acceptance','0'),('log','log','0')]
ARMS={'off':'0','on':'1'}


def capture(out,group,scene,hl,pair,arm,seconds):
    binary=Path.home()/'.cache/rustui-target/release/rustui-lab'
    stem=f'{group}-{pair}-{arm}';log=out/(stem+'.log')
    flags=dict(os.environ,RUSTUI_G5_REFERENCE='1',RUSTUI_DAMAGE='0',RUSTUI_FRAME_LOG='buffered',
               RUSTUI_FRAME_PROFILE='1',RUSTUI_G0_LATENCY='1',RUSTUI_LATENCY_BREAKDOWN='1',RUSTUI_G5_REFRESH='1',
               RUSTUI_HL_FASTPATH=hl,GPUI_FAST_IMMEDIATE_PRESENT=ARMS[arm],RUSTUI_DRAW_FIRST=ARMS[arm],
               GPUI_FAST_IMMEDIATE_COALESCE=ARMS[arm],GPUI_FAST_IMMEDIATE_ANY_CHANGE=ARMS[arm],
               RUSTUI_G0_TYPING_HZ='30' if group=='typing-30' else '10',GPUI_FAST_VIEW_DISPLAY_LINK='0',GPUI_FAST_FORCE_LEGACY_LINK='0',
               GPUI_FAST_FRAME_AHEAD='0',GPUI_FAST_IMMEDIATE_HOLD_MS='250',GPUI_FAST_IMMEDIATE_TRANSACTION='0',
               RUSTUI_F0_PANEL='collection',DYLD_INSERT_LIBRARIES=str(ROOT/'target/headstart/g5-1/libG5Refresh.dylib'))
    for key in ['RUSTUI_G0_HOVER_PIXELS','RUSTUI_G5_PIXEL_CAPTURE','MTL_HUD_ENABLED','RUSTUI_F0_SCROLL','RUSTUI_G5_OVERLAY']:
        flags.pop(key,None)
    print(stem,'start',datetime.now(timezone.utc).isoformat(),flush=True)
    if log.exists():
        def saved_load(phase):
            path=out/(stem+'.top-'+phase+'.txt');meta=json.loads(Path(str(path)+'.processes.json').read_text())
            processes={int(k):v for k,v in meta['processes'].items()}
            return metrics.background_load(path.read_text(),processes,meta['logical_cpus'],meta['measured'],meta['collector'],meta['limit'])
        before,after=saved_load('start'),saved_load('end')
        print(stem,'reusing saved raw capture',flush=True)
    else:
        before=metrics.collect_load(str(out/(stem+'.top-start.txt')),[str(binary)],None)
        (out/(stem+'.top-five.json')).write_text(json.dumps(before['top_processes'],indent=2)+'\n')
        with log.open('w') as stream:
            subprocess.run([str(binary),'--g0-scene',scene,'--duration',str(seconds)],cwd=ROOT,env=flags,stdout=stream,stderr=subprocess.STDOUT,check=True,timeout=seconds+45)
        after=metrics.collect_load(str(out/(stem+'.top-end.txt')),[str(binary)],None)
    records=metrics.records(log);x=metrics.bare(records,2)
    metrics.annotate_load(x,before,after)
    assert x['final_text_valid'],(stem,'correctness',x['out_of_order_presentations'])
    assert x['last_version_shown']==x['expected_final_version'],(stem,'last version',x['last_version_shown'],x['expected_final_version'])
    events=records['RUSTUI_PX4_LOG '];latest={};violations=[];counts=[]
    for event in events:
        if event.get('kind')=='g5_acquire_start':latest[event['window']]=event
        if event.get('kind')=='next_drawable':
            pre=latest.get(event['window']);assert pre is not None
            count,pool,_,attached=pre['values'];counts.append(count)
            if arm=='on' and not(attached==0 and count<pool):
                violations.append(pre)
    assert not violations,(stem,violations)
    markers=records['RUSTUI_G0_LOG '];start=next(r for r in markers if r['event']=='start')['mach'];end=next(r for r in markers if r['event']=='end')['mach']
    changes=[r for r in events if r.get('kind')=='g53_kit_change_queued' and start<=r['mach']<=end]
    if scene=='typing':
        revisions=[r['values'][1] for r in changes]
        x['lost_native_edits']=x['inputs']-len(revisions)
        assert x['lost_native_edits']==0 and all(b==a+1 for a,b in zip(revisions,revisions[1:])),(stem,'native edit revisions',len(revisions),x['inputs'])
    x['same_scene_skips']=sum(r.get('kind')=='g53_surface_coalesced' and r['values'][2]==1 and start<=r['mach']<=end for r in events)
    x['refresh_deferrals']=sum(r.get('kind')=='g53_surface_coalesced' and r['values'][2]==2 and start<=r['mach']<=end for r in events)
    from collections import Counter
    causes=Counter()
    for e in events:
        if e.get('kind')=='g53_frame_cause' and start<=e['mach']<=end:
            mask=int(e['values'][0])
            # A draw can consume several causes. Keep both the exact masks and
            # overlapping cause totals; never pretend these are exclusive.
            causes['mask_'+str(mask)]+=1
            for bit,name in [(1,'input'),(2,'binding'),(4,'colour')]:
                if mask&bit:causes[name]+=1
            if not mask:causes['other']+=1
    x['frame_causes']=dict(causes)
    x['cpu_build_deferrals']=sum(e.get('kind')=='g53_cpu_build_deferred' and start<=e['mach']<=end for e in events)
    if scene=='acceptance':
        current={};offsets={};attempts={};commits={};displayed=[]
        clock=next(e for e in events if e['event']=='clock');scale=clock['numer']/clock['denom']/1e9
        for e in events:
            w=e.get('window')
            if e['event']=='draw':current[w]=e['draw']
            elif e.get('kind')=='g53_list_paint_offset' and w in current:offsets[(w,current[w])]=tuple(e['values'][:2])
            elif e['event']=='attempt':attempts[(w,e['attempt'])]=e['draw']
            elif e['event']=='committed':commits[(w,e['drawable'])]=e['attempt']
            elif e['event']=='displayed' and e['presented_host']>0:displayed.append((e['presented_host'],w,e['drawable']))
        witnessed=[]
        for when,w,drawable in sorted(displayed):
            attempt=commits.get((w,drawable));draw=attempts.get((w,attempt));offset=offsets.get((w,draw))
            if offset is not None:witnessed.append((w,offset,when))
        slots=metrics.visible_change_slots(witnessed,start*scale,end*scale)
        x['visible_versions_per_second']=slots['visible_change_slots']/x['seconds']
        x['scroll_offset_witnessed_presentations']=slots['presentation_slots']
        x['ambiguous_scroll_slots']=slots['ambiguous_presented_slots']
    x['cpu_s_per_s']=(x['cpu_user_s']+x['cpu_sys_s'])/x['seconds']
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
    parser.add_argument('--resume',action='store_true')
    args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    paths=[Path.home()/'.cache/rustui-target/release/rustui-lab',ROOT/'target/headstart/g5-1/libG5Refresh.dylib']
    frozen={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    frozen_path=args.output/'frozen-build.json'
    if args.resume:assert json.loads(frozen_path.read_text())==frozen,'runtime changed before resume'
    else:frozen_path.write_text(json.dumps(frozen,indent=2)+'\n')
    deadline=datetime.fromisoformat(args.deadline.replace('Z','+00:00')).timestamp();rows=json.loads((args.output/'captures.json').read_text()) if args.resume else []
    done={(r['group'],r['pair'],r['arm']) for r in rows}
    try:
        for group,scene,hl in GROUPS:
            for pair in range(1,2 if args.functional else 6):
                arms=['on'] if args.functional else (['off','on'] if pair%2 else ['on','off'])
                arms=[arm for arm in arms if (group,pair,arm) not in done]
                if not arms:continue
                needed=sum(0 if (args.output/f'{group}-{pair}-{arm}.log').exists() else (17 if args.functional else 70) for arm in arms)
                if time.time()+needed>deadline:
                    print('Timebox: no new matched pair started.',flush=True);return 75
                for arm in arms:
                    rows.append(capture(args.output,group,scene,hl,pair,arm,5 if args.functional else 60))
                    (args.output/'captures.json').write_text(json.dumps(rows,indent=2)+'\n')
        return 0
    finally:
        current={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
        (args.output/'frozen-build-after.json').write_text(json.dumps(current,indent=2)+'\n')
        assert current==frozen,'executables changed during timing'

if __name__=='__main__':raise SystemExit(main())
