"""G5 local capture analysis. Never substitute submit/callback time for display."""
from bisect import bisect_left, bisect_right
from collections import Counter
import re
import os
import subprocess
import argparse
import json
import math
from pathlib import Path
import statistics


def distribution(values):
    if not values:
        return {"samples": 0, "median": None, "p95": None, "max": None}
    values = sorted(values)
    return {"samples": len(values), "median": statistics.median(values),
            "p95": values[math.ceil(.95 * len(values)) - 1], "max": values[-1]}


def refresh_index(times, timestamp):
    i = bisect_left(times, timestamp)
    choices = [j for j in (i - 1, i) if 0 <= j < len(times)]
    return min(choices, key=lambda j: abs(times[j] - timestamp))


def refresh_metrics(refreshes, pairs, floor_frames):
    times = sorted(set(refreshes))
    intervals = [(b - a) * 1000 for a, b in zip(times, times[1:]) if b > a]
    if len(intervals) < 10:
        return {"observed_refresh_hz": None, "missed_refreshes": distribution([]),
                "control_floor_candidates": {}, "floor_frames": floor_frames, "raw_missed_refreshes": []}
    period = statistics.median(intervals) / 1000
    candidates, missed = [], []
    for gpu, displayed in pairs:
        eligible = bisect_left(times, gpu)
        if eligible == 0 or eligible >= len(times) - 1 or displayed > times[-1] + period / 2:
            continue
        shown = refresh_index(times, displayed)
        # Reject an uncalibrated or implausibly distant display clock.
        if abs(times[shown] - displayed) > period / 2:
            continue
        candidates.append(shown - eligible)
        if floor_frames is not None:
            missed.append(max(0, shown - eligible - floor_frames))
    return {"observed_refresh_hz": 1 / period, "refresh_intervals_ms": distribution(intervals),
            "missed_refreshes": distribution(missed),
            "control_floor_candidates": dict(Counter(candidates)), "floor_frames": floor_frames, "raw_missed_refreshes": missed}


def p_verdict(changes):
    assert len(changes) == 5 and all(math.isfinite(v) for v in changes)
    if all(v <= -3 for v in changes):
        return "GAIN"
    if all(v > 3 for v in changes):
        return "REGRESSION"
    if all(-3 <= v <= 3 for v in changes):
        return "NEUTRAL"
    return "INCONCLUSIVE"


def records(path):
    prefixes = ("G5_REF ", "RUSTUI_PX4_LOG ", "RUSTUI_G0_LOG ",
                "RUSTUI_FRAME_LOG ", "G5_REFRESH ", "G5_PIXELS ")
    out = {p: [] for p in prefixes}
    for line in Path(path).read_text().splitlines():
        for prefix in prefixes:
            if line.startswith(prefix):
                out[prefix].append(json.loads(line[len(prefix):]))
                break
    return out


def reference(data, floor_frames):
    events = data["G5_REF "]
    start = next(r for r in events if r["event"] == "start")
    end = next(r for r in events if r["event"] == "end")
    assert end["valid"], "reference final text differs from expected"
    scale = start["numer"] / start["denom"] / 1e9
    seconds = (end["mach"] - start["mach"]) * scale
    inputs = {r["version"]: r["mach"] for r in events if r["event"] == "handled"
              and start["mach"] <= r["mach"] <= end["mach"]}
    completed = {r["version"]: r for r in events if r["event"] == "completed"}
    displayed = []
    is_metal = "transaction" in start
    for r in events:
        if is_metal and r["event"] == "presented" and r["presented"] > 0:
            shown = r["mach"] * scale - (r["host"] - r["presented"])
            displayed.append((shown, r["version"], r["presented"]))
        elif not is_metal and r["event"] == "displayed_pixels":
            displayed.append((r["display_mach"] * scale, r["version"], None))
    shown_versions, delays, pairs, observations = [], [], [], []
    first_input = min(inputs.values(), default=end["mach"]) * scale
    for shown, version, host in sorted(displayed):
        if shown >= first_input:
            observations.append(version)
        if version not in inputs:
            continue
        assert shown >= inputs[version] * scale, "display predates input"
        if version not in shown_versions:
            delays.append((shown - inputs[version] * scale) * 1000)
            shown_versions.append(version)
        if host is not None and version in completed and completed[version]["gpu_end"] > 0:
            pairs.append((completed[version]["gpu_end"], host))
    acquisition = [r["ms"] for r in events if r["event"] == "next_drawable"]
    refreshes = [r["timestamp"] for r in events if r["event"] == "cadence"]
    result = {"method": "MTLDrawable.presentedTime" if is_metal else "decoded pixels + WindowServer displayTime",
              "seconds": seconds, "inputs": len(inputs), "shown": len(shown_versions),
              "distinct_versions_per_second": len(shown_versions) / seconds,
              "input_to_display_ms": distribution(delays), "raw_input_to_display_ms": delays,
              "gpu_complete_to_display_ms": distribution([(d - g) * 1000 for g, d in pairs]),
              "rendered_never_shown": len(inputs) - len(shown_versions) if is_metal else None,
              "rendered_versions_without_capture": len(inputs) - len(shown_versions),
              "versions_monotonic": observations == sorted(observations),
              "final_text_valid": end["valid"], "last_version_shown": observations[-1] if observations else 0,
              "next_drawable_calls": len(acquisition), "next_drawable_over_1_ms": sum(v > 1 for v in acquisition),
              "next_drawable_ms": distribution(acquisition), "gpu_display_pairs": pairs,
              "cpu_s": end.get("cpu_s"), "cpu_s_per_attempt": end.get("cpu_s", 0) / len(acquisition) if acquisition else None}
    result.update(refresh_metrics(refreshes, pairs, floor_frames))
    return result


def acquisition_checks(events, start, end):
    calls = [r for r in events if r.get("kind") == "next_drawable"]
    points = [r for r in events if r.get("kind") == "g5_acquire_start"]
    outliers = []
    for call in calls:
        if call["values"][0] <= 1:
            continue
        before = next((r for r in reversed(points) if r["window"] == call["window"] and r["mach"] <= call["mach"]), None)
        clock = next((r for r in events if r.get("event") == "clock"), None)
        scale = clock["numer"] / clock["denom"] / 1e6 if clock else None
        tick = before["mach"] if before else call["mach"] - call["values"][0] / scale if scale else call["mach"]
        detach = next((r for r in reversed(events) if r.get("window") == call["window"] and r.get("kind") == "g5_link_stop" and r["mach"] <= tick), None)
        outliers.append({"mach": call["mach"], "ms": call["values"][0],
                         "in_flight": before["values"][0] if before else call["values"][1],
                         "in_flight_sample": "pre-call" if before else "post-call",
                         "time_since_detach_ms": (tick-detach["mach"])*scale if detach and scale else None,
                         "phase": "warmup" if call["mach"] < start else "drain" if call["mach"] > end else "measured",
                         "path": "immediate" if call["values"][3] else "non-immediate",
                         "pre_acquire_state": before["values"] if before else None})
    return {"next_drawable_calls_all": len(calls),
            "next_drawable_over_1_ms_all": len(outliers),
            "next_drawable_outliers": outliers}


def callback_presentation_metrics(events, displays, start, end, scale):
    callbacks = {(r["window"], r["callback"]): r["mach"] for r in events if r["event"] == "callback"}
    callback_delays = [(r["mach"] - callbacks[(r["window"], r["callback"])]) * scale * 1000
                       for r in events if r["event"] == "draw" and start <= r["mach"] <= end
                       and (r["window"], r["callback"]) in callbacks]
    shown_times = {}
    for shown_mach, row, _, _ in displays:
        if start <= shown_mach <= end:
            shown_times.setdefault(row["window"], set()).add(shown_mach)
    gaps = [(b-a)*scale*1000 for times in shown_times.values()
            for a,b in zip(sorted(times), sorted(times)[1:])]
    return {"tick_callback_to_draw_ms": distribution(callback_delays),
            "presentation_gaps_ms": distribution(gaps),
            "presentation_gap_histogram_ms": dict(Counter(round(gap, 1) for gap in gaps))}


def presentation_order_violations(observations):
    highest = {}
    count = 0
    for window, version in observations:
        previous = highest.get(window, -1)
        count += version < previous
        highest[window] = max(previous, version)
    return count


def surface_commits(events):
    current, result = {}, {}
    for event in sorted(events, key=lambda r:r.get("mach",0)):
        window = event.get("window")
        if event.get("kind") == "g5_surface_submit":
            current[window] = tuple(int(v) for v in event["values"][:2])
        elif event.get("event") == "committed":
            result[(window,event["drawable"])] = current.get(window,(0,None))
    return result


def rendered_frame_counts(events, displays, start, end):
    # Count submitted drawable frames, including repeated GPU renders of a
    # reused CPU scene. A CPU draw ID is not a drawable/frame identity.
    committed = {(r["window"], r["drawable"]) for r in events
                 if r.get("event") == "committed" and start <= r["mach"] <= end}
    shown = {(r["window"], r["drawable"]) for _, r, _, _ in displays}
    return {"rendered": len(committed),
            "rendered_never_shown": len(committed - shown)}


def bare(data, floor_frames):
    events, markers = data["RUSTUI_PX4_LOG "], data["RUSTUI_G0_LOG "]
    start = next(r for r in markers if r["event"] == "start")
    end = next(r for r in markers if r["event"] == "end")
    assert end["valid"] and end.get("final_version", end["ticks"]) == end["ticks"], "lost edit / wrong final text"
    clock = next(r for r in events if r["event"] == "clock")
    scale = clock["numer"] / clock["denom"] / 1e9
    seconds = (end["mach"] - start["mach"]) * scale
    inputs = {(r["window"], r["version"]): r for r in events if r["event"] == "handled"
              and start["mach"] <= r["mach"] <= end["mach"]}
    keyed = lambda kind, key: {(r["window"], r[key]): r for r in events if r["event"] == kind}
    attempts = keyed("attempt", "attempt")
    paints = keyed("painted", "draw")
    commits = keyed("committed", "drawable")
    surfaces = surface_commits(events)
    completed = keyed("gpu_completed", "drawable")
    draws = {(r["window"], r["draw"]) for r in events if r["event"] == "draw"
             and start["mach"] <= r["mach"] <= end["mach"]}
    displays, invalid = [], 0
    for r in events:
        if r["event"] != "displayed":
            continue
        p, host = r["presented_host"], r["callback_host"]
        if not (0 < p <= host and math.isfinite(p) and math.isfinite(host)):
            invalid += 1
            continue
        commit = commits.get((r["window"], r["drawable"]))
        if commit is None:
            continue
        attempt = attempts.get((r["window"], commit["attempt"]))
        if attempt is None:
            continue
        paint = paints.get((r["window"], attempt["draw"]))
        if paint is None:
            continue
        shown_mach = r["callback_mach"] - (host - p) / scale
        displays.append((shown_mach, r, attempt, paint))
    shown_inputs, shown_draws, delays, pairs, versions = set(), set(), [], [], []
    version_observations = []
    first_frames = []
    first_input = min((r["mach"] for r in inputs.values()), default=end["mach"])
    for shown_mach, r, attempt, paint in sorted(displays, key=lambda item: item[0]):
        if shown_mach >= first_input:
            versions.append(paint["version"])
            version_observations.append(((r["window"],surfaces.get((r["window"],r["drawable"]),(0,None))[0]),paint["version"]))
        key = (r["window"], paint["version"])
        inp = inputs.get(key)
        draw_key = (r["window"], attempt["draw"])
        if draw_key in draws:
            shown_draws.add(draw_key)
        if inp is None or shown_mach < inp["mach"]:
            continue
        if key not in shown_inputs:
            shown_inputs.add(key)
            delays.append((shown_mach - inp["mach"]) * scale * 1000)
            first_frames.append((inp, shown_mach, r, attempt, paint))
        gpu = completed.get((r["window"], r["drawable"]))
        if gpu is not None and gpu["gpu_end_host"] > 0:
            pairs.append((gpu["gpu_end_host"], r["presented_host"]))
    configs = [r for r in events if r["event"] == "config" and start["mach"] <= r["mach"] <= end["mach"]]
    acquisitions = [r["values"][0] for r in configs if r["kind"] == "next_drawable"]
    timed = [r for r in data["RUSTUI_FRAME_LOG "] if start["epoch_ms"] <= r["epoch_ms"] <= end["epoch_ms"]]
    count = sum(r["event"] == "present" for r in timed)
    frames, last = [], 0
    for r in timed:
        if r["event"] == "draw":
            last = r["cpu_ms"]
        elif r["event"] == "present":
            frames.append(last + r["cpu_ms"])
            last = 0
    cpu = end["cpu_user_s"] + end["cpu_sys_s"]
    assert count > 0 and len(inputs) == end["ticks"]
    result = {"method": "MTLDrawable.presentedTime", "seconds": seconds, "inputs": len(inputs),
              "attempts": count, "shown": len(shown_inputs), "distinct_versions_per_second": len(shown_inputs) / seconds,
              "input_to_display_ms": distribution(delays), "raw_input_to_display_ms": delays,
              "gpu_complete_to_display_ms": distribution([(d - g) * 1000 for g, d in pairs]),
              "rendered_cpu_draws": len(draws), "scene_draws_without_presentation": len(draws - shown_draws),
              "versions_monotonic": presentation_order_violations(version_observations) == 0,
              "out_of_order_presentations": presentation_order_violations(version_observations), "final_text_valid": end["valid"],
              "last_version_shown": versions[-1] if versions else 0, "expected_final_version": end["ticks"],
              "unavailable_presented_callbacks": invalid,
              "next_drawable_calls": len(acquisitions), "next_drawable_over_1_ms": sum(v > 1 for v in acquisitions),
              "next_drawable_ms": distribution(acquisitions), "gpu_display_pairs": pairs,
              "cpu_user_s": end["cpu_user_s"], "cpu_sys_s": end["cpu_sys_s"],
              "cpu_s_per_attempt": cpu / count, "draw_present_ms_per_attempt": statistics.mean(frames),
              "frames_over_8_3_ms": sum(v > 8.3 for v in frames),
              "queue_count_max": max((r["values"][0] for r in configs if r["kind"] == "g5_queue"), default=None)}
    result.update(rendered_frame_counts(events, displays, start["mach"], end["mach"]))
    result.update(acquisition_checks(events, start["mach"], end["mach"]))
    result.update(refresh_metrics([r["timestamp"] for r in data["G5_REFRESH "]], pairs, floor_frames))
    result.update(classify_inputs(events, first_frames, commits, result))
    result["screen_capture"] = pixel_metrics(data["G5_PIXELS "], inputs, scale,
                                           first_frames, seconds)
    result.update(callback_presentation_metrics(events, displays, start["mach"], end["mach"], scale))
    result["next_drawable_over_2_ms"] = sum(v > 2 for v in acquisitions)
    result["frame_source_selections"] = [r["values"] for r in events if r["event"] == "config" and r["kind"] == "g5_frame_source"]
    return result


def classify_inputs(events, first_frames, commits, result):
    hz = result.get("observed_refresh_hz")
    median = result["input_to_display_ms"]["median"]
    threshold = median + 1000 / hz if hz and median is not None else None
    queues = sorted((r for r in events if r["event"] == "config" and r["kind"] == "g5_queue"), key=lambda r: r["mach"])
    states = sorted((r for r in events if r["event"] == "config" and r["kind"] == "g5_input_state"), key=lambda r: r["mach"])
    lifecycle = [r for r in events if r["event"] == "config" and r["kind"] in ("g5_link_stop", "g5_link_recreate")]
    # The handled marker is stamped by fast_measure_input before dispatch.
    # The matching dispatch-entry state is the first following snapshot.
    indices = {}
    for rows in (queues, states):
        grouped = {}
        for row in rows:grouped.setdefault(row["window"], []).append(row)
        indices[id(rows)] = {window: ([r["mach"] for r in group], group) for window,group in grouped.items()}
    def previous(rows, window, tick):
        times, group = indices[id(rows)].get(window, ([], []))
        i=bisect_right(times,tick)-1
        return group[i] if i>=0 else None
    def following(rows, window, tick):
        times, group = indices[id(rows)].get(window, ([], []))
        i=bisect_left(times,tick)
        return group[i] if i<len(group) else None
    clock = next(r for r in events if r["event"] == "clock")
    slow, classification = [], Counter()
    ordered = sorted(first_frames, key=lambda item: item[0]["mach"])
    previous_tick = 0
    for inp, shown_mach, display, attempt, paint in ordered:
        commit = commits[(display["window"], display["drawable"])]
        queue = previous(queues, display["window"], commit["mach"])
        path = ("immediate" if queue["values"][2] else "display-link") if queue and queue["mach"] >= attempt["mach"] else "unknown"
        state = following(states, inp["window"], inp["mach"])
        # Mach conversion uses the same calibration as bare().
        delay = (shown_mach - inp["mach"]) * clock["numer"] / clock["denom"] / 1e6
        classification[path] += 1
        if threshold is not None and delay > threshold:
            row = {"event": "g5_slow_input", "window": inp["window"], "version": paint["version"], "input_mach": inp["mach"],
                   "display_mach": shown_mach, "delay_ms": delay, "threshold_ms": threshold,
                   "path": path, "input_state_mach": state["mach"] if state else None, "attached_at_input": bool(state["values"][0]) if state else None,
                   "in_flight_at_input": state["values"][1] if state else None,
                   "pool_size_at_input": state["values"][2] if state else None,
                   "held_at_input": bool(state["values"][3]) if state else None,
                   "link_events": [{"event": r["kind"], "mach": r["mach"]} for r in lifecycle
                                   if r["window"] == inp["window"] and previous_tick <= r["mach"] <= shown_mach]}
            slow.append(row)
        previous_tick = inp["mach"]
    return {"slow_threshold_ms": threshold, "slow_inputs": slow, "input_paths": dict(classification),
            "link_lifecycle_counts": dict(Counter(r["kind"] for r in lifecycle))}


def pixel_metrics(pixel_rows, inputs, scale, first_frames, seconds):
    by_version = {version: inp for (_, version), inp in inputs.items()}
    positive_present = {paint["version"]: shown for _, shown, _, _, paint in first_frames}
    seen, delays, deltas, observations = set(), [], [], []
    first_input = min((r["mach"] for r in by_version.values()), default=float("inf"))
    for row in sorted(pixel_rows, key=lambda r: r["display_mach"]):
        version = row["version"]
        if row["display_mach"] >= first_input:
            observations.append(version)
        if version in seen or version not in by_version:
            continue
        inp = by_version[version]
        assert row["display_mach"] >= inp["mach"], "captured version predates its input"
        seen.add(version)
        delays.append((row["display_mach"] - inp["mach"]) * scale * 1000)
        if version in positive_present:
            deltas.append((row["display_mach"] - positive_present[version]) * scale * 1000)
    return {"method": "decoded pixels + WindowServer displayTime", "input_to_display_ms": distribution(delays),
            "raw_input_to_display_ms": delays, "captured_versions": len(seen), "inputs": len(by_version),
            "distinct_versions_per_second": len(seen) / seconds,
            "versions_monotonic": observations == sorted(observations),
            "last_version_captured": observations[-1] if observations else 0,
            "displayTime_minus_presentedTime_ms": distribution(deltas)}


def background_cpu_percent(text):
    """Read total host user+system CPU before applying process exclusions."""
    samples = re.findall(r"CPU usage: ([\d.]+)% user, ([\d.]+)% sys, ([\d.]+)% idle", text)
    if not samples:
        raise ValueError("top did not report a CPU summary")
    user, system, _ = map(float, samples[-1])
    return user + system


def process_snapshot():
    rows = subprocess.check_output(["ps", "-Ao", "pid=,ppid=,comm="], text=True)
    result = {}
    for line in rows.splitlines():
        fields = line.strip().split(None, 2)
        if len(fields) == 3:
            result[int(fields[0])] = {"ppid": int(fields[1]), "path": fields[2]}
    return result


def background_load(text, processes, logical_cpus, measured=(), collector=None, limit=10):
    """Subtract only proven Codex/measured descendants in the same top interval."""
    if limit not in (10, 25, None):
        raise ValueError("background limit must be 10, 25 or owner-authorized none")
    if logical_cpus < 1 or text.count("CPU usage:") < 2:
        raise ValueError("need logical CPU count and a completed top interval")
    total = background_cpu_percent(text)
    last = text[text.rfind("CPU usage:"):]
    rows = []
    for line in last.splitlines():
        match = re.match(r"^\s*(\d+)\s+(\d+)\s+([\d.]+)\s+(.+)$", line)
        if match:
            pid, parent, cpu, name = match.groups()
            rows.append({"pid": int(pid), "ppid": int(parent), "cpu_per_core": float(cpu),
                         "path": processes.get(int(pid), {}).get("path", name.strip())})
    if not rows:
        raise ValueError("top did not report process CPU rows")
    # Include sampled children (notably top itself) that exited before ps.
    processes = dict(processes)
    for row in rows:
        processes.setdefault(row["pid"], {"ppid": row["ppid"], "path": row["path"]})
    measured = {str(Path(p).resolve()) for p in measured}
    roots = {pid for pid, p in processes.items()
             if "/ChatGPT.app/" in p["path"] or "/Codex.app/" in p["path"]
             or p["path"] in measured}
    if collector is not None:
        roots.add(collector)
    excluded = set(roots)
    while True:
        children = {pid for pid, p in processes.items() if p["ppid"] in excluded}
        if children <= excluded:
            break
        excluded |= children
    for row in rows:
        row["excluded"] = row["pid"] in excluded
    excluded_cpu = sum(r["cpu_per_core"] for r in rows if r["excluded"]) / logical_cpus
    background = max(0., total-excluded_cpu)
    consistent = excluded_cpu <= total
    rows.sort(key=lambda r: r["cpu_per_core"], reverse=True)
    return {"total_cpu_percent": total, "excluded_cpu_percent": excluded_cpu,
            "background_cpu_percent": background, "logical_cpus": logical_cpus,
            "consistent": consistent, "limit": limit,
            "accepted": consistent and (limit is None or background <= limit),
            "top_processes": rows[:5], "top_background_processes": [r for r in rows if not r["excluded"]][:15],
            "excluded_processes": [r for r in rows if r["excluded"] and r["cpu_per_core"] > 0]}


def collect_load(path, measured, limit=10):
    before = process_snapshot()
    result = subprocess.run(["top", "-l", "2", "-s", "1", "-o", "cpu", "-n", "100000",
                             "-stats", "pid,ppid,cpu,command"], env=dict(os.environ, LC_ALL="C"),
                            text=True, capture_output=True, check=True)
    processes = before | process_snapshot()
    Path(path).write_text(result.stdout)
    meta = {"processes": processes, "logical_cpus": os.cpu_count(), "collector": os.getpid(), "measured": measured, "limit": limit}
    Path(str(path)+".processes.json").write_text(json.dumps(meta, indent=2))
    return background_load(result.stdout, processes, meta["logical_cpus"], measured, meta["collector"], limit)


def annotate_load(result, start, end):
    limit = start.get("limit", 10)
    if limit != end.get("limit", 10) or not start["accepted"] or not end["accepted"]:
        raise ValueError("capture requires matching accepted load checks")
    a, b = start["background_cpu_percent"], end["background_cpu_percent"]
    label = (f"measured on a loaded machine ({a:.2f}% background at start; {b:.2f}% after)"
             if limit in (25, None) else f"measured with background CPU {a:.2f}% at start; {b:.2f}% after")
    result["machine_load"] = {"limit": limit, "start_background_cpu_percent": a,
                              "end_background_cpu_percent": b, "label": label,
                              "owner_authorized_bypass": limit is None}
    return result


def summarize(path, floor_frames=None):
    data = records(path)
    return reference(data, floor_frames) if data["G5_REF "] else bare(data, floor_frames)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture")
    parser.add_argument("--floor-frames", type=int)
    parser.add_argument("--slow-output")
    parser.add_argument("--check-load", action="store_true")
    parser.add_argument("--collect-load", action="store_true")
    parser.add_argument("--measured-executable", action="append", default=[])
    parser.add_argument("--background-limit", choices=("10", "25", "none"), default="10",
                        help="none requires an explicit owner-authorized CPU gate bypass")
    parser.add_argument("--start-load")
    parser.add_argument("--end-load")
    args = parser.parse_args()
    if args.collect_load or args.check_load:
        if args.collect_load:
            limit = None if args.background_limit == "none" else int(args.background_limit)
            result = collect_load(args.capture, args.measured_executable, limit)
        else:
            meta = json.loads(Path(args.capture+".processes.json").read_text())
            result = background_load(Path(args.capture).read_text(), {int(k): v for k, v in meta["processes"].items()},
                                     meta["logical_cpus"], meta["measured"], meta["collector"], meta.get("limit", 10))
        Path(args.capture+".json").write_text(json.dumps(result, indent=2))
        print(json.dumps(result, indent=2))
        raise SystemExit(0 if result["accepted"] else 75)
    result = summarize(args.capture, args.floor_frames)
    if args.start_load or args.end_load:
        if not args.start_load or not args.end_load:
            parser.error("both --start-load and --end-load are required")
        result = annotate_load(result, json.loads(Path(args.start_load).read_text()), json.loads(Path(args.end_load).read_text()))
    if args.slow_output:
        Path(args.slow_output).write_text("".join(json.dumps(row)+"\n" for row in result.get("slow_inputs", [])))
    print(json.dumps(result, indent=2))
