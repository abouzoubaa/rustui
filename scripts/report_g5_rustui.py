#!/usr/bin/env python3
"""Render the G5-3 matched capture evidence without inventing incomplete verdicts."""
import argparse,json,statistics
from collections import Counter
from pathlib import Path
from g5_latency import p_verdict
from run_g5_rustui import GROUPS


def fmt(value):return 'NOT MEASURED' if value is None else f'{value:.3f}'
def med(values):
    values=[v for v in values if v is not None]
    return statistics.median(values) if values else None

def render(rows):
    text='## Captures\n\nAll numbers below are measured on a loaded machine. Quantiles are medians of capture quantiles; maxima are maxima across captures. Missing valid presented callbacks are not proof that pixels were physically absent.\n\n'
    text+='| workload | arm | captures / 5 | input→display median / p95 / max ms | GPU→display median ms | missed refreshes median / p95 | versions/s median (min) | presentation gaps median / p95 / max ms | unmatched / rendered | nextDrawable >1 / >2 ms | process CPU ms/attempt | CPU frames >8.3 ms | max pre-acquire live |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for group,_,_ in GROUPS:
        for arm in ('off','immediate','reserve'):
            part=sorted([r for r in rows if r['group']==group and r['arm']==arm],key=lambda r:r['pair'])
            if not part:continue
            d=lambda key,field='median':med([r[key][field] for r in part])
            latency='/'.join(fmt(v) for v in (d('input_to_display_ms'),d('input_to_display_ms','p95'),max(r['input_to_display_ms']['max'] for r in part)))
            gaps='/'.join(fmt(v) for v in (d('presentation_gaps_ms'),d('presentation_gaps_ms','p95'),max(r['presentation_gaps_ms']['max'] or 0 for r in part)))
            rates=[r['distinct_versions_per_second'] for r in part]
            target=(f"; below 110: {sum(v<110 for v in rates)}/{len(rates)}" if group in ('hover','drag') else '')
            text+=f"| {group} | {arm} | {len(part)} | {latency} | {fmt(d('gpu_complete_to_display_ms'))} | {fmt(d('missed_refreshes'))}/{fmt(d('missed_refreshes','p95'))} | {fmt(med(rates))} ({fmt(min(rates))}){target} | {gaps} | {sum(r['rendered_never_shown'] for r in part)}/{sum(r['rendered'] for r in part)} | {sum(r['next_drawable_over_1_ms'] for r in part)}/{sum(r['next_drawable_over_2_ms'] for r in part)} | {fmt(1000*med([r['cpu_s_per_attempt'] for r in part]))} | {sum(r['frames_over_8_3_ms'] for r in part)} | {max(r['max_pre_acquire_in_flight'] for r in part)} |\n"
    text+='\n## Paired CPU verdicts and reserve-arm effects\n\nNegative paired CPU change means less process CPU per attempted present. Attempt counts may change between arms: a normalized GAIN does not itself mean lower total process CPU or lower latency. The per-capture table shows total CPU and the denominator. A complete set requires all five paired captures. Latency/cadence comparisons have no invented percentage tolerance.\n\n| workload | comparison | five paired process CPU changes % | median / min / max % | P verdict |\n| --- | --- | --- | --- | --- |\n'
    conclusions=[]
    for group,_,_ in GROUPS:
        for off,on in ([('off','immediate'),('immediate','reserve')] if group in ('hover','drag') else [('off','immediate')]):
            a={r['pair']:r for r in rows if r['group']==group and r['arm']==off};b={r['pair']:r for r in rows if r['group']==group and r['arm']==on};pairs=sorted(a.keys()&b.keys())
            changes=[100*(b[p]['cpu_s_per_attempt']/a[p]['cpu_s_per_attempt']-1) for p in pairs]
            complete=pairs==[1,2,3,4,5]
            verdict=p_verdict(changes) if complete else 'INCOMPLETE'
            stats='/'.join(fmt(v) for v in (med(changes),min(changes,default=None),max(changes,default=None)))
            text+=f"| {group} | {off}→{on} | {', '.join(fmt(v) for v in changes) or 'none'} | {stats} | {verdict} |\n"
            if on=='reserve':
                if not complete:conclusions.append(f'- {group}: reserve comparison incomplete; no conclusion.');continue
                median_d=[b[p]['input_to_display_ms']['median']-a[p]['input_to_display_ms']['median'] for p in pairs]
                p95_d=[b[p]['input_to_display_ms']['p95']-a[p]['input_to_display_ms']['p95'] for p in pairs]
                rate_d=[b[p]['distinct_versions_per_second']-a[p]['distinct_versions_per_second'] for p in pairs]
                waits=sum(b[p]['next_drawable_over_2_ms'] for p in pairs)
                claim=('All five reserve captures had no >2 ms acquisitions, no higher median/p95 and no lower shown-version rate.' if waits==0 and all(v<=0 for v in median_d+p95_d) and all(v>=0 for v in rate_d) else 'The requested no-waits/no-latency-rise/no-cadence-loss combination is not established across all five pairs.')
                conclusions.append(f"- {group}: reserve measured >2 ms calls={waits}; paired median-delay changes ms [{', '.join(fmt(v) for v in median_d)}]; p95 changes [{', '.join(fmt(v) for v in p95_d)}]; versions/s changes [{', '.join(fmt(v) for v in rate_d)}]. {claim}")
    text+='\n'+'\n'.join(conclusions)+'\n'
    text+='\n## Observed-refresh missed-slot diagnosis\n\nEach count is one observed refresh with no new version. The inferential limits and inherited control floor are described above.\n\n| workload | arm | observed slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps >1.5 refreshes |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for group in ('hover','drag'):
        for arm in ('off','immediate','reserve'):
            part=[r['missed_slots'] for r in rows if r['group']==group and r['arm']==arm and 'missed_slots' in r]
            if not part:continue
            counts=Counter()
            for r in part:counts.update(r['counts'])
            text+=f"| {group} | {arm} | {sum(r['observed_slots'] for r in part)} | "+' | '.join(str(counts[k]) for k in ['no new input','frame not ready','no drawable free','presented but replaced','WindowServer late'])+f" | {sum(r['observer_gaps_over_1_5_refreshes'] for r in part)} |\n"
    text+='\n## Per-capture diagnostics\n\n| workload | pair | arm | attempts | CPU draws / GPU renders | user / sys / total CPU seconds | process CPU ms/attempt | draw+present ms/attempt | observed refresh Hz | background start→end % |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n'
    for r in rows:
        load=r['machine_load']
        text+=f"| {r['group']} | {r['pair']} | {r['arm']} | {r['attempts']} | {r['rendered_cpu_draws']}/{r['rendered']} | {fmt(r['cpu_user_s'])}/{fmt(r['cpu_sys_s'])}/{fmt(r['cpu_user_s']+r['cpu_sys_s'])} | {fmt(r['cpu_s_per_attempt']*1000)} | {fmt(r['draw_present_ms_per_attempt'])} | {fmt(r['observed_refresh_hz'])} | {fmt(load['start_background_cpu_percent'])}→{fmt(load['end_background_cpu_percent'])} |\n"
    text+='\n## Every acquisition over 1 ms\n\nW/M/D denote warmup/measured/drain. Each entry gives phase, path, duration ms, pre-call live/pool, attached state and milliseconds since detach (N/A = no previous detach).\n'
    for r in rows:
        if not r['next_drawable_outliers']:continue
        values=[]
        for v in r['next_drawable_outliers']:
            pre=v['pre_acquire_state'];age='N/A' if v['time_since_detach_ms'] is None else f"{v['time_since_detach_ms']:.3f}"
            values.append(f"{v['phase'][0].upper()},{v['path']},{v['ms']:.6f},{int(pre[0])}/{int(pre[1])},{bool(pre[3])},{age}")
        text+=f"\n- {r['group']} pair {r['pair']} {r['arm']}: `"+'; '.join(values)+'`\n'
    text+='\nAll behavior switches remain off by default. No adoption is inferred from incomplete, NEUTRAL or INCONCLUSIVE CPU results.\n'
    return text


def main():
    p=argparse.ArgumentParser();p.add_argument('captures',type=Path);p.add_argument('--document',type=Path,required=True);a=p.parse_args()
    rows=json.loads(a.captures.read_text())
    original=a.document.read_text().split('## Captures')[0].replace('Results pending the frozen-build matched captures. Incomplete sets receive no P verdict.','Incomplete sets receive no P verdict.')
    a.document.write_text(original.rstrip()+'\n\n'+render(rows))

if __name__=='__main__':main()
