#!/usr/bin/env python3
"""Render G5-3b evidence; total process CPU/second decides the P verdict."""
import argparse,json,statistics
from collections import Counter
from pathlib import Path
from g5_latency import p_verdict
from run_g5_rustui import GROUPS


def fmt(v):return 'NOT MEASURED' if v is None else f'{v:.3f}'
def med(v):return statistics.median(v) if v else None

def render(rows):
    text='## G5-3b matched captures\n\nQuantiles below are medians of the five capture quantiles; maxima are maxima across captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.\n\n'
    text+='| workload | arm | captures / 5 | input→display median / p95 / max ms | visible versions/s (lower bound) | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | CPU draw+present >8.3 ms |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for group,_,_ in GROUPS:
        for arm in ('off','on'):
            part=[r for r in rows if r['group']==group and r['arm']==arm]
            if not part:continue
            q=lambda field:med([r['input_to_display_ms'][field] for r in part])
            rates=[r.get('visible_versions_per_second',r['distinct_versions_per_second']) for r in part]
            text+=f"| {group} | {arm} | {len(part)} | {fmt(q('median'))}/{fmt(q('p95'))}/{fmt(max(r['input_to_display_ms']['max'] for r in part))} | {fmt(med(rates))} | {sum(r['rendered_never_shown'] for r in part)}/{sum(r['rendered'] for r in part)} | {sum(r['out_of_order_presentations'] for r in part)} | {sum(r.get('lost_native_edits',0) for r in part)} | {sum(r['next_drawable_over_2_ms'] for r in part)} | {fmt(med([r['cpu_s_per_s'] for r in part]))} | {fmt(1000*med([r['cpu_s_per_attempt'] for r in part]))} | {fmt(med([r['draw_present_ms_per_attempt'] for r in part]))} | {sum(r['frames_over_8_3_ms'] for r in part)} |\n"
    text+='\n## Paired CPU gate\n\nDeciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.\n\n| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |\n| --- | --- | --- | --- | --- |\n'
    for group,_,_ in GROUPS:
        a={r['pair']:r for r in rows if r['group']==group and r['arm']=='off'};b={r['pair']:r for r in rows if r['group']==group and r['arm']=='on'};pairs=sorted(a.keys()&b.keys())
        changes=[100*(b[p]['cpu_s_per_s']/a[p]['cpu_s_per_s']-1) for p in pairs]
        diagnostic=[100*(b[p]['cpu_s_per_attempt']/a[p]['cpu_s_per_attempt']-1) for p in pairs]
        verdict=p_verdict(changes) if pairs==[1,2,3,4,5] else 'INCOMPLETE'
        stats='/'.join(fmt(v) for v in [med(changes),min(changes,default=None),max(changes,default=None)])
        text+=f"| {group} | {', '.join(fmt(v) for v in changes) or 'none'} | {stats} | {verdict} | {', '.join(fmt(v) for v in diagnostic)} |\n"
    text+='\n## Frame and refresh diagnostics\n\nThroughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.\n\nThe same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.\n\n| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for group,_,_ in GROUPS:
        for arm in ('off','on'):
            part=[r for r in rows if r['group']==group and r['arm']==arm]
            if not part:continue
            q=lambda key,field='median':med([r[key][field] for r in part if r[key][field] is not None])
            text+=f"| {group} | {arm} | {sum(r['rendered_cpu_draws'] for r in part)}/{sum(r['rendered'] for r in part)}/{sum(r['inputs'] for r in part)} | {fmt(q('gpu_complete_to_display_ms'))} | {fmt(q('missed_refreshes'))}/{fmt(q('missed_refreshes','p95'))} | {fmt(q('presentation_gaps_ms'))}/{fmt(q('presentation_gaps_ms','p95'))}/{fmt(max(r['presentation_gaps_ms']['max'] or 0 for r in part))} | {sum(r.get('same_scene_skips',0) for r in part)} | {sum(r.get('refresh_deferrals',0) for r in part)} | {sum(r.get('ambiguous_presented_slots',0) for r in part)} |\n"
    text+='\n## Observed-refresh missed slots\n\nThese labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.\n\n| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for group in ('hover','drag'):
        for arm in ('off','on'):
            part=[r['missed_slots'] for r in rows if r['group']==group and r['arm']==arm and 'missed_slots' in r]
            if not part:continue
            counts=Counter()
            for r in part:counts.update(r['counts'])
            text+=f"| {group} | {arm} | {sum(r['observed_slots'] for r in part)} | "+' | '.join(str(counts[k]) for k in ['no new input','frame not ready','no drawable free','presented but replaced','WindowServer late'])+f" | {sum(r['observer_gaps_over_1_5_refreshes'] for r in part)} |\n"
    text+='\n## Per-capture process CPU and load\n\nEach loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.\n\n| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for r in rows:
        load=r['machine_load'];before=load['start_background_cpu_percent'];after=load['end_background_cpu_percent'];label='loaded' if max(before,after)>10 else 'quiet'
        text+=f"| {r['group']} | {r['pair']} | {r['arm']} | {r['attempts']} | {fmt(r['seconds'])} | {fmt(r['cpu_user_s'])}/{fmt(r['cpu_sys_s'])}/{fmt(r['cpu_user_s']+r['cpu_sys_s'])} | {fmt(r['cpu_s_per_s'])} | {fmt(1000*r['cpu_s_per_attempt'])} | {fmt(r['observed_refresh_hz'])} | {fmt(before)}→{fmt(after)} | {label} |\n"
    text+='\n## Known acquisitions over 2 ms\n\nAll phases are listed. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.\n'
    count=0
    for r in rows:
        for v in r['next_drawable_outliers']:
            if v['ms']<=2:continue
            count+=1;pre=v['pre_acquire_state'];age='N/A' if v['time_since_detach_ms'] is None else fmt(v['time_since_detach_ms'])
            text+=f"\n- {r['group']} pair {r['pair']} {r['arm']}: {v['phase']}, {v['path']}, {v['ms']:.6f} ms, {int(pre[0])}/{int(pre[1])}, attached={bool(pre[3])}, detach age={age} ms.\n"
    if not count:text+='\nNone observed in these captures.\n'
    return text

def main():
    p=argparse.ArgumentParser();p.add_argument('captures',type=Path);p.add_argument('--document',type=Path,required=True);a=p.parse_args()
    rows=json.loads(a.captures.read_text());original=a.document.read_text().split('## G5-3b matched captures')[0]
    a.document.write_text(original.rstrip()+'\n\n'+render(rows))
if __name__=='__main__':main()
