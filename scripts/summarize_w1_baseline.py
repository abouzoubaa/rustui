#!/usr/bin/env python3
"""Recompute the three-session W1 calibration from preserved evidence.

Specific to the recorded W1 protocol; outputs descriptive repeatability, never
an optimization acceptance decision. Native trace summaries are supplied evidence.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

from runtime_metrics import write_json_exclusive
from summarize_runtime_metrics import read_record, summarize_record


def analyze(root):
    native_path = root / 'w1-calibration-native.json'
    native = read_record(native_path)
    if native.get('kind') != 'rustui_w1_native_calibration':
        raise ValueError('native execution evidence missing')
    expected_trace = []
    for step in range(7):
        prefix = 'Parent · ' if step == 3 else ''
        children = [[prefix + 'Alpha', int(step >= 1)], [prefix + 'Beta', int(step >= 2)]]
        if step == 5:
            children.reverse()
        expected_trace.append({'step': step, 'children': children})
    if json.dumps(native.get('unique_observed_traces'), sort_keys=True) != json.dumps([expected_trace], sort_keys=True):
        raise ValueError('native child observations do not match W1')
    sessions = []
    identities = []
    evidence = [native_path]
    for number in (1, 2, 3):
        manifest_path = root / f'w1-calibration-session{number}-manifest.json'
        active_path = root / f'process-w1-calibration-active{number}.json'
        idle_path = root / f'process-w1-calibration-idle{number}.json'
        manifest = read_record(manifest_path)
        if manifest.get('profile') not in ('dev', 'headstart', 'release'):
            raise ValueError('unsupported W1 build profile')
        for field, expected in (('session', number), ('warmup_cycles', 3), ('measured_cycles', 30)):
            if type(manifest.get(field)) is not int or manifest[field] != expected:
                raise ValueError('manifest W1 protocol mismatch: ' + field)
        active_record, idle_record = read_record(active_path), read_record(idle_path)
        active, idle = summarize_record(active_record), summarize_record(idle_record)
        execution = next(s for s in native['sessions'] if s['session'] == number)
        for field in ('session', 'warmup_cycles', 'measured_cycles', 'verified_states'):
            if type(execution.get(field)) is not int:
                raise ValueError('native W1 protocol requires integer ' + field)
        if (execution['warmup_cycles'] != 3 or execution['measured_cycles'] != 30
                or execution['verified_states'] != 210
                or execution['all_cycles_match_unique_trace'] is not True):
            raise ValueError('incomplete native W1 execution evidence')
        for record, summary in ((active_record, active), (idle_record, idle)):
            if summary['completion_status'] != 'complete' or summary['pid'] != manifest['pid']:
                raise ValueError('process evidence incomplete or PID mismatch')
        if active_record['identity'] != idle_record['identity']:
            raise ValueError('process identity changed between idle and active capture')
        if active_record.get('termination') != 'stop_file':
            raise ValueError('active collection did not end at requested boundary')
        if active_record['binary_reference']['sha256'] != manifest['sha256']:
            raise ValueError('binary references differ')
        identities.append({k: manifest[k] for k in (
            'sha256', 'source_hashes', 'viewport_logical', 'appearance', 'locale',
            'profile', 'warmup_cycles', 'measured_cycles', 'sampling_interval_seconds')})
        idle_cpu_rate = idle['cpu_total_delta_ns'] / idle['observed_sample_elapsed_ns']
        adjusted_cpu_ns = active['cpu_total_delta_ns'] - idle_cpu_rate * active['observed_sample_elapsed_ns']
        sessions.append({
            'session': number, 'pid': manifest['pid'], 'active': active, 'idle': idle,
            'gross_cpu_seconds_per_cycle': active['cpu_total_delta_ns'] / 1e9 / 30,
            'idle_adjusted_cpu_seconds_per_cycle_estimate': adjusted_cpu_ns / 1e9 / 30,
            'rss_median_mib': active['rss_bytes']['median'] / 2**20,
            'rss_max_mib': active['rss_bytes']['max'] / 2**20,
            'rss_last_minus_first_bytes': active_record['samples'][-1]['rss_bytes'] - active_record['samples'][0]['rss_bytes'],
            'execution': execution,
        })
        evidence.extend((manifest_path, active_path, idle_path))
    if any(identity != identities[0] for identity in identities[1:]):
        raise ValueError('build or protocol differs between launches')
    values = [s['gross_cpu_seconds_per_cycle'] for s in sessions]
    median = statistics.median(values)
    adjusted = [s['idle_adjusted_cpu_seconds_per_cycle_estimate'] for s in sessions]
    return {
        'schema_version': 1, 'kind': 'rustui_w1_baseline_calibration',
        'purpose': 'same-build CPU/RSS baseline and observed repeatability; no candidate verdict',
        'measured_cycles': 90, 'verified_states': 630, 'sessions': sessions,
        'common_inputs': identities[0],
        'repeatability': {
            'cpu_seconds_per_cycle_median': median,
            'cpu_seconds_per_cycle_min': min(values),
            'cpu_seconds_per_cycle_max': max(values),
            'cpu_range_percent_of_median': (max(values)-min(values)) / median * 100 if median > 0 else None,
            'cpu_sample_cv_percent': statistics.stdev(values) / statistics.mean(values) * 100 if statistics.mean(values) > 0 else None,
            'idle_adjusted_cpu_seconds_per_cycle_median_estimate': statistics.median(adjusted),
            'idle_adjusted_range_percent_of_median_estimate': (max(adjusted)-min(adjusted))/statistics.median(adjusted)*100 if statistics.median(adjusted) > 0 else None,
        },
        'limitations': [
            f"Build profile: {identities[0]['profile']}; host, power and background context are recorded in each session manifest.",
            'Gross CPU includes inter-batch idle. The separately labeled idle-adjusted estimate subtracts each launch idle CPU rate across the full capture; it assumes stationary background cost and is not an exact measurement.',
            'Native automation durations include tool/AX overhead and are not application cycle or frame latency.',
            'ps CPU display resolution is 10 ms here; converted ns do not imply ns precision.',
            'RSS is resident memory, not physical footprint or GPU memory.',
            'Three-launch spread is descriptive, not a confidence interval or approved regression threshold.',
            'Frame, main-thread, GPU and presentation timing and optimized-candidate comparisons are not measured.',
            'Native execution summary is an exported observation record; this script does not rerun the GUI.',
        ],
        'evidence_sha256': {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in evidence},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('target/headstart/performance'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    write_json_exclusive(args.output, analyze(args.root))


if __name__ == '__main__':
    main()
