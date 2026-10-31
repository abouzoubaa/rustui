import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import summarize_w1_baseline as summary
from scripts.tests.test_summarize_runtime_metrics import record


def fixture(root, profile='release', zero_cpu=False):
    trace = []
    for step in range(7):
        prefix = 'Parent · ' if step == 3 else ''
        children = [[prefix + 'Alpha', int(step >= 1)], [prefix + 'Beta', int(step >= 2)]]
        trace.append({'step': step, 'children': children[::-1] if step == 5 else children})
    native = {'kind': 'rustui_w1_native_calibration', 'unique_observed_traces': [trace], 'sessions': []}
    for number in (1, 2, 3):
        native['sessions'].append(dict(session=number, warmup_cycles=3, measured_cycles=30,
                                       verified_states=210, all_cycles_match_unique_trace=True))
        manifest = dict(session=number, pid=123 + number, sha256='a' * 64,
                        source_hashes={'app': 'b' * 64}, viewport_logical=[1320, 820],
                        appearance='macos27-dark', locale='en', profile=profile,
                        warmup_cycles=3, measured_cycles=30, sampling_interval_seconds=0.25)
        (root / f'w1-calibration-session{number}-manifest.json').write_text(json.dumps(manifest))
        for phase in ('idle', 'active'):
            data = record()
            data['pid'] = data['identity']['pid'] = manifest['pid']
            data['binary_reference'] = {'sha256': manifest['sha256']}
            data['termination'] = 'stop_file' if phase == 'active' else 'duration'
            if phase == 'idle' or zero_cpu:
                for i, sample in enumerate(data['samples']):
                    sample['cpu_time_ns'] = 0 if zero_cpu else i * 10_000_000
            (root / f'process-w1-calibration-{phase}{number}.json').write_text(json.dumps(data))
    (root / 'w1-calibration-native.json').write_text(json.dumps(native))


class W1SummaryTests(unittest.TestCase):
    def test_reports_actual_profile_for_development_and_release(self):
        for profile in ('dev', 'headstart', 'release'):
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture(root, profile)
                result = summary.analyze(root)
                self.assertEqual(result['common_inputs']['profile'], profile)
                self.assertIn('Build profile: ' + profile, result['limitations'][0])
                self.assertNotIn('release-build', ' '.join(result['limitations']))
                self.assertEqual(result['measured_cycles'], 90)

    def test_rejects_manifest_cycle_denominator_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root)
            path = root / 'w1-calibration-session1-manifest.json'
            data = json.loads(path.read_text())
            data['measured_cycles'] = 20
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, 'manifest W1 protocol mismatch'):
                summary.analyze(root)

    def test_rejects_boolean_and_float_native_observations(self):
        for count in (True, 1.0):
            with self.subTest(count=count), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture(root)
                path = root / 'w1-calibration-native.json'
                data = json.loads(path.read_text())
                data['unique_observed_traces'][0][1]['children'][0][1] = count
                path.write_text(json.dumps(data))
                with self.assertRaisesRegex(ValueError, 'native child observations'):
                    summary.analyze(root)

    def test_zero_cpu_keeps_relative_statistics_unavailable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root, zero_cpu=True)
            stats = summary.analyze(root)['repeatability']
            self.assertEqual(stats['cpu_seconds_per_cycle_median'], 0)
            for key in ('cpu_range_percent_of_median', 'cpu_sample_cv_percent',
                        'idle_adjusted_range_percent_of_median_estimate'):
                self.assertIsNone(stats[key])

    def test_rejects_unknown_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture(root, 'unrecorded')
            with self.assertRaisesRegex(ValueError, 'unsupported W1 build profile'):
                summary.analyze(root)


if __name__ == '__main__':
    unittest.main()
