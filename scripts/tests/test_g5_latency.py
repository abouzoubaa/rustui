import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from g5_latency import distribution, p_verdict, refresh_metrics, classify_inputs, pixel_metrics, background_cpu_percent, acquisition_checks, background_load, annotate_load, reference, callback_presentation_metrics, rendered_frame_counts, presentation_order_violations, surface_commits, visible_change_slots


class LatencyTests(unittest.TestCase):
    def test_order_violations_count_stale_frames_against_each_windows_high_water(self):
        self.assertEqual(presentation_order_violations([('w',8),('w',6),('z',1),('w',7)]),2)
        self.assertEqual(presentation_order_violations([('w',6),('z',1),('w',7),('z',1),('w',8)]),0)

    def test_equal_presented_time_does_not_invent_order_from_clock_conversion_jitter(self):
        self.assertEqual(presentation_order_violations([('w',49,1.0),('w',48,1.0),('w',50,2.0)]),0)
        self.assertEqual(presentation_order_violations([('w',8,1.0),('w',6,2.0),('w',7,3.0)]),2)

    def test_same_time_versions_occupy_one_visible_slot_and_overlap_is_unknown(self):
        result=visible_change_slots([('w',1,1.),('w',2,1.),('w',2,2.),('w',3,3.)],0.,4.)
        self.assertEqual(result,{'visible_change_slots':2,'ambiguous_presented_slots':1,'presentation_slots':3})

    def test_composition_order_is_checked_per_surface(self):
        events=[{"event":"config","kind":"g5_surface_submit","window":"w","values":[1,8,1,1],"mach":1},
                {"event":"committed","window":"w","drawable":10,"mach":2},
                {"event":"config","kind":"g5_surface_submit","window":"w","values":[2,6,0,1],"mach":3},
                {"event":"committed","window":"w","drawable":11,"mach":4}]
        self.assertEqual(surface_commits(events),{("w",10):(1,8),("w",11):(2,6)})
        self.assertEqual(presentation_order_violations([(("w",1),8),(("w",2),6),(("w",2),7)]),0)
        self.assertEqual(presentation_order_violations([(("w",1),8),(("w",1),6),(("w",1),7)]),2)

    def test_rendered_frames_include_repeated_presents_of_one_cpu_draw(self):
        events = [
            {"event":"committed","window":"w","drawable":1,"draw":9,"mach":10},
            {"event":"committed","window":"w","drawable":2,"draw":9,"mach":20},
            {"event":"committed","window":"w","drawable":3,"draw":10,"mach":30},
            {"event":"committed","window":"w","drawable":4,"draw":10,"mach":40}]
        displays = [(50,{"window":"w","drawable":1},None,None),
                    (60,{"window":"w","drawable":4},None,None)]
        self.assertEqual(rendered_frame_counts(events,displays,10,30),
                         {"rendered":3,"rendered_never_shown":2})

    def test_callback_delay_excludes_immediate_frames_and_gaps_are_per_window(self):
        events = [{"event":"callback","window":"w","callback":1,"mach":100},
                  {"event":"draw","window":"w","callback":1,"mach":300},
                  {"event":"draw","window":"w","callback":0,"mach":400}]
        displays = [(tick,{"window":window},None,None) for window,tick in
                    [('w',1000),('z',1500),('w',2000),('z',2500),('w',20000)]]
        result=callback_presentation_metrics(events,displays,0,10000,1e-6)
        self.assertEqual(result['tick_callback_to_draw_ms']['samples'],1)
        self.assertAlmostEqual(result['tick_callback_to_draw_ms']['median'],.2)
        self.assertEqual(result['presentation_gaps_ms']['samples'],2)
        self.assertAlmostEqual(result['presentation_gaps_ms']['median'],1)

    def test_reference_preserves_redisplayed_versions_in_both_endpoints(self):
        for metal in [False, True]:
            start = {"event": "start", "mach": 0, "numer": 1, "denom": 1}
            if metal:
                start["transaction"] = False
            events = [start, {"event": "handled", "version": 1, "mach": 1000},
                      {"event": "handled", "version": 2, "mach": 2000},
                      {"event": "end", "mach": 1000000, "valid": True}]
            for version, tick in [(1, 3000), (2, 4000), (1, 5000)]:
                events.append({"event": "presented", "version": version, "mach": tick,
                               "host": tick * 1e-9, "presented": tick * 1e-9} if metal else
                              {"event": "displayed_pixels", "version": version, "display_mach": tick})
            result = reference({"G5_REF ": events}, None)
            self.assertFalse(result["versions_monotonic"])
            self.assertEqual(result["last_version_shown"], 1)
            self.assertEqual(result["shown"], 2)
            self.assertEqual(result["input_to_display_ms"]["samples"], 2)

    def test_pixel_regression_keeps_distinct_latency_and_actual_last_version(self):
        rows = [{"version": v, "display_mach": t} for v, t in
                [(1, 3000), (2, 4000), (1, 5000), (0, 6000)]]
        inputs = {('w', 1): {"mach": 1000}, ('w', 2): {"mach": 2000}}
        result = pixel_metrics(rows, inputs, 1e-9, [], 1)
        self.assertFalse(result["versions_monotonic"])
        self.assertEqual(result["last_version_captured"], 0)
        self.assertEqual(result["captured_versions"], 2)
        self.assertEqual(result["input_to_display_ms"]["samples"], 2)

    def test_acquisition_checks_keep_warmup_and_exact_pre_call_state(self):
        events = [{"event": "clock", "numer": 1, "denom": 1},
                  {"kind": "g5_link_stop", "window": "w", "mach": 1},
                  {"kind": "g5_acquire_start", "window": "w", "mach": 10, "values": [0, 3, 0, 0]},
                  {"kind": "next_drawable", "window": "w", "mach": 20, "values": [2, 0, 3, 0]},
                  {"kind": "g5_acquire_start", "window": "w", "mach": 40, "values": [2, 3, 1, 0]},
                  {"kind": "next_drawable", "window": "w", "mach": 50, "values": [5, 2, 3, 1]}]
        result = acquisition_checks(events, 30, 100)
        self.assertEqual(result["next_drawable_over_1_ms_all"], 2)
        self.assertEqual([r["phase"] for r in result["next_drawable_outliers"]], ["warmup", "measured"])
        self.assertEqual(result["next_drawable_outliers"][1]["pre_acquire_state"], [2, 3, 1, 0])
        self.assertEqual(result["next_drawable_outliers"][1]["in_flight"], 2)
        self.assertAlmostEqual(result["next_drawable_outliers"][1]["time_since_detach_ms"], .000039)

    def test_quiet_gate_subtracts_codex_descendants_and_measured_once(self):
        processes = {1: {"ppid": 0, "path": "/sbin/launchd"},
                     10: {"ppid": 40, "path": "/Applications/ChatGPT.app/Contents/MacOS/codex"},
                     11: {"ppid": 10, "path": "/bin/zsh"},
                     12: {"ppid": 11, "path": "/tmp/latency-gpui"},
                     20: {"ppid": 1, "path": "/usr/bin/python3 -m graphify.watch"},
                     40: {"ppid": 1, "path": "/Applications/Code.app/Contents/MacOS/Code"}}
        text = ("CPU usage: 1.0% user, 1.0% sys, 98.0% idle\n"
                "CPU usage: 25.0% user, 15.0% sys, 60.0% idle\nPID PPID %CPU COMMAND\n"
                "10 40 100.0 codex\n11 10 50.0 zsh\n12 11 50.0 latency-gpui\n"
                "20 1 100.0 python3\n40 1 20.0 Code\n77 11 10.0 top\n")
        result = background_load(text, processes, 10, ["/tmp/latency-gpui"], collector=11)
        self.assertEqual(result["excluded_cpu_percent"], 21)
        self.assertEqual(result["background_cpu_percent"], 19)
        self.assertFalse(result["accepted"])
        self.assertEqual({r["pid"] for r in result["top_background_processes"]}, {20, 40})
        self.assertEqual(len(result["top_processes"]), 5)

    def test_corrected_quiet_gate_has_inclusive_ten_percent_limit(self):
        def sample(total):
            return background_load(f"CPU usage: 0.0% user, 0.0% sys, 100.0% idle\nCPU usage: {total}% user, 0.0% sys, 0.0% idle\n1 0 0.0 launchd\n", {}, 10)
        self.assertTrue(sample(10)["accepted"])
        self.assertFalse(sample(10.1)["accepted"])

    def test_corrected_quiet_gate_rejects_missing_or_inconsistent_sample(self):
        with self.assertRaises(ValueError):
            background_load("CPU usage: 9.0% user, 1.0% sys, 90.0% idle", {}, 10)
        text = "CPU usage: 0.0% user, 0.0% sys, 100.0% idle\nCPU usage: 1.0% user, 1.0% sys, 98.0% idle\n10 1 100.0 codex\n"
        result = background_load(text, {10: {"ppid": 1, "path": "/Applications/Codex.app/Contents/MacOS/Codex"}}, 10)
        self.assertFalse(result["consistent"])
        self.assertFalse(result["accepted"])

    def test_owner_loaded_limit_is_opt_in_and_bounded(self):
        text = "CPU usage: 0.0% user, 0.0% sys, 100.0% idle\nCPU usage: 25.0% user, 0.0% sys, 75.0% idle\n1 0 0.0 launchd\n"
        self.assertFalse(background_load(text, {}, 10)["accepted"])
        self.assertTrue(background_load(text, {}, 10, limit=25)["accepted"])
        self.assertFalse(background_load(text.replace("25.0% user", "25.1% user"), {}, 10, limit=25)["accepted"])
        with self.assertRaises(ValueError):
            background_load(text, {}, 10, limit=99)

    def test_loaded_capture_label_preserves_background_values(self):
        a = {"limit": 25, "accepted": True, "background_cpu_percent": 21.23}
        b = {"limit": 25, "accepted": True, "background_cpu_percent": 24.56}
        result = annotate_load({"cpu_s_per_attempt": .003}, a, b)
        self.assertEqual(result["cpu_s_per_attempt"], .003)
        self.assertIn("measured on a loaded machine (21.23% background", result["machine_load"]["label"])
        self.assertEqual(result["machine_load"]["end_background_cpu_percent"], 24.56)
        with self.assertRaises(ValueError):
            annotate_load({}, a, dict(b, accepted=False))

    def test_owner_bypass_keeps_load_measurement_and_default_gate(self):
        text = "CPU usage: 0.0% user, 0.0% sys, 100.0% idle\nCPU usage: 80.0% user, 0.0% sys, 20.0% idle\n1 0 0.0 launchd\n"
        self.assertFalse(background_load(text, {}, 10)["accepted"])
        result = background_load(text, {}, 10, limit=None)
        self.assertTrue(result["accepted"])
        self.assertEqual(result["background_cpu_percent"], 80)
        labeled = annotate_load({}, result, result)["machine_load"]
        self.assertTrue(labeled["owner_authorized_bypass"])
        self.assertIn("measured on a loaded machine (80.00% background", labeled["label"])
        inconsistent = text.replace("80.0% user", "1.0% user").replace("1 0 0.0 launchd", "10 1 100.0 codex")
        processes = {10: {"ppid": 1, "path": "/Applications/Codex.app/Contents/MacOS/Codex"}}
        self.assertFalse(background_load(inconsistent, processes, 10, limit=None)["accepted"])

    def test_quiet_gate_uses_total_cpu_and_fails_closed(self):
        self.assertEqual(background_cpu_percent("CPU usage: 9.0% user, 1.0% sys, 90.0% idle"), 10)
        self.assertGreater(background_cpu_percent("CPU usage: 9.0% user, 1.1% sys, 89.9% idle"), 10)
        with self.assertRaises(ValueError):
            background_cpu_percent("unrecognized top output")

    def test_fixed_control_floor_excludes_floor_but_counts_later_refresh(self):
        times = [i / 120 for i in range(100)]
        # GPU completes just before refresh 20; floor is one extra refresh.
        result = refresh_metrics(times, [(19.5 / 120, 21 / 120), (29.5 / 120, 33 / 120)], 1)
        self.assertEqual(result['missed_refreshes']['median'], 1)
        self.assertEqual(result['missed_refreshes']['p95'], 2)
        self.assertAlmostEqual(result['observed_refresh_hz'], 120)
        self.assertEqual(result['control_floor_candidates'], {1: 1, 3: 1})

    def test_missing_floor_is_not_zero_misses(self):
        result = refresh_metrics([i / 120 for i in range(100)], [(10.1 / 120, 14 / 120)], None)
        self.assertIsNone(result['missed_refreshes']['median'])
        self.assertIsNone(distribution([])['median'])

    def test_p_rule_uses_all_five_changes_and_inclusive_boundaries(self):
        self.assertEqual(p_verdict([-3] * 5), 'GAIN')
        self.assertEqual(p_verdict([3] * 5), 'NEUTRAL')
        self.assertEqual(p_verdict([3.001] * 5), 'REGRESSION')
        self.assertEqual(p_verdict([-4, -4, -4, -4, 0]), 'INCONCLUSIVE')

    def test_slow_input_uses_input_state_and_actual_frame_path(self):
        inp = {"window": "w", "version": 1, "mach": 1_000_000_000}
        attempt = {"mach": 1_001_000_000}
        display = {"window": "w", "drawable": 9}
        events = [{"event": "clock", "numer": 1, "denom": 1},
                  {"event": "config", "kind": "g5_input_state", "window": "w", "mach": 999_999_999, "values": [0, 0, 3, 0]},
                  {"event": "config", "kind": "g5_input_state", "window": "w", "mach": 1_000_000_001, "values": [1, 2, 3, 0]},
                  {"event": "config", "kind": "g5_queue", "window": "w", "mach": 1_002_000_000, "values": [2, 3, 0, 0]},
                  {"event": "config", "kind": "g5_link_recreate", "window": "w", "mach": 1_003_000_000}]
        result = classify_inputs(events, [(inp, 1_040_000_000, display, attempt, {"version": 1})],
                                 {("w", 9): {"mach": 1_004_000_000}},
                                 {"observed_refresh_hz": 120, "input_to_display_ms": {"median": 20}})
        slow = result['slow_inputs'][0]
        self.assertEqual(slow['path'], 'display-link')
        self.assertTrue(slow['attached_at_input'])
        self.assertEqual(slow['input_state_mach'], 1_000_000_001)
        self.assertEqual(slow['in_flight_at_input'], 2)
        self.assertEqual(slow['link_events'][0]['event'], 'g5_link_recreate')

    def test_pixels_use_display_time_and_observed_version_not_callback(self):
        rows = [{"version": 1, "display_mach": 3000, "callback_mach": 90000},
                {"version": 1, "display_mach": 4000, "callback_mach": 100000},
                {"version": 9, "display_mach": 5000, "callback_mach": 100000}]
        result = pixel_metrics(rows, {('w', 1): {"mach": 1000}}, 1e-9,
                               [(None, 4000, None, None, {"version": 1})], 1)
        self.assertEqual(result['captured_versions'], 1)
        self.assertAlmostEqual(result['input_to_display_ms']['median'], .002)
        self.assertAlmostEqual(result['displayTime_minus_presentedTime_ms']['median'], -.001)


if __name__ == '__main__':
    unittest.main()
