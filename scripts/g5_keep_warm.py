#!/usr/bin/env python3
"""Run and summarize opt-in G5-L5 native display-link comparisons."""
import argparse
from bisect import bisect_left
from collections import Counter
import csv
from datetime import datetime, timezone
import json
from pathlib import Path
import os
import shutil
import subprocess
import sys
import time

from g5_latency import collect_load, distribution, p_verdict

APPS = ("lab", "bare", "appkit")
MODES = ("off", "4", "250ms")
SCK_MISS_SECONDS = 0.008333
RATE_TOLERANCE_SECONDS = 0.0005
QUIET_LIMIT = 5.0


def records(path, prefix):
    with path.open() as log:
        return [json.loads(line[len(prefix):]) for line in log if line.startswith(prefix)]


def update_intervals(times, arrival):
    """Return the last completed interval, its age, and the interval straddling arrival."""
    index = bisect_left(times, arrival)
    previous = times[index-1] - times[index-2] if index >= 2 else None
    age = arrival - times[index-1] if index else None
    around = times[index] - times[index-1] if 0 < index < len(times) else None
    return previous, age, around


def active_link_interval(timestamps, targets, arrival):
    index = bisect_left(timestamps, arrival) - 1
    if index >= 0 and timestamps[index] <= arrival <= targets[index]:
        return targets[index] - timestamps[index]
    return None


def overlapping_late_ticks(intervals, start, end):
    return [(ready, actual) for ready, actual in intervals if ready < end and actual >= start]


def refresh_group(interval, age):
    # The sampled timestamp tracks framebuffer updates. A stale timestamp
    # or a large gap can include unchanged refreshes, so it is not a panel rate.
    if interval is None or age is None or age > .025 or interval > .025:
        return "unobserved"
    if interval > 1 / 120 + RATE_TOLERANCE_SECONDS:
        return "slower_update"
    if interval < 1 / 120 - RATE_TOLERANCE_SECONDS:
        return "other_interval"
    return "near_120_update"


def inspect_capture(directory, app, mode):
    summary = json.loads((directory / "summary.json").read_text())
    path = directory / "capture.log"
    native = records(path, "L1_NATIVE ")
    clock = next(row for row in native if row["event"] == "clock")
    seconds_per_tick = clock["numer"] / clock["denom"] / 1e9
    inputs = {r["version"]: r for r in native if r["event"] == "native"}
    seen = {r["version"]: r for r in summary["rows"]}

    if app != "appkit":
        markers = records(path, "RUSTUI_G0_LOG ")
        start = next(r["mach"] for r in markers if r["event"] == "start")
        end = next(r["mach"] for r in markers if r["event"] == "end")
        px4 = records(path, "RUSTUI_PX4_LOG ")
        configs = sorted([r for r in px4 if r.get("kind") == "metal_layer"], key=lambda r: r["mach"])
        before = next(r for r in reversed(configs) if r["mach"] <= start)
        assert all(r["values"][1] == 0 for r in [before] + [r for r in configs if start < r["mach"] <= end]), path
        # This is existing evidence of the GPUI render link, only when attached.
        render_links = [r for r in px4 if r.get("kind") == "display_link"]
    else:
        render_links = []

    notes = records(path, "G5_L5 ")
    assert notes and any(r["event"] == "dropped" and r["count"] == 0 for r in notes), path
    pinned = [r["values"] for r in notes if r["event"] == "pinned"]
    unsupported = any(r["event"] == "unsupported" for r in notes)
    nsscreen_rows = {int(r["values"][0]): r["values"] for r in notes if r["event"] == "key_screen"}
    screen_updates = sorted({r["values"][0] for r in notes if r["event"] == "screen_update"})
    link = sorted({(r["values"][0], r["values"][1]) for r in notes if r["event"] == "link_tick"})
    link_timestamps, link_targets = tuple(zip(*link)) if link else ((), ())
    pauses = [r["values"] for r in notes if r["event"] == "paused"]
    stalls = [(r["values"][0], r["values"][1]) for r in notes if r["event"] == "main_late"]
    missed_stalls = set()
    if mode == "off":
        assert not pinned and not link and not pauses, path
    elif not unsupported:
        assert pinned and link and all(p[0] == p[1] == p[2] > 0 for p in pinned), path
        if mode == "4":
            assert pauses and all(p[0] == 4 for p in pauses), path
        if mode == "250ms":
            assert all(p[1] >= p[2] for p in pauses), path

    sck_rows = records(path, "G5_SCK_CADENCE ")
    assert sck_rows, path
    # Idle deliveries may repeat a compositor timestamp. Keep the distinct,
    # positive timestamps and the observed status counts separately.
    times = sorted({r["display_mach"] * seconds_per_tick for r in sck_rows})

    rows = []
    for version in range(1, summary["inputs"] + 1):
        n = inputs[version]
        measured = seen.get(version, {})
        key_time = n["hardware_s"]
        handler = n["handler_mach"] * seconds_per_tick
        display, gpu = measured.get("display"), measured.get("gpu")
        eligible = gpu is not None and display is not None
        margin = display - gpu if eligible else None
        if app == "appkit":
            key_class = "no_gpu_clock"
        elif not eligible:
            key_class = "unobserved"
        elif margin < 0:
            key_class = "negative_clock_margin"
        else:
            key_class = "miss" if margin >= SCK_MISS_SECONDS else "hit"
        sck_previous, sck_age, sck_around = update_intervals(times, key_time)
        previous, age, around = update_intervals(screen_updates, key_time)
        link_seconds = active_link_interval(link_timestamps, link_targets, key_time)
        key_screen = nsscreen_rows.get(version)
        overlaps = overlapping_late_ticks(stalls, key_time, display) if display is not None else []
        if key_class == "miss":
            missed_stalls.update(overlaps)
        rows.append(dict(
            capture=directory.name, app=app, mode=mode, version=version, classification=key_class,
            key_host_s=key_time, handler_host_s=handler, sck_host_s=display, gpu_end_host_s=gpu,
            handler_sck_ms=measured.get("sck_total"), P_ms=measured.get("P"),
            gpu_sck_ms=margin * 1000 if margin is not None else None,
            sck_prior_s=sck_previous, sck_prior_age_s=sck_age, sck_bracketing_s=sck_around,
            screen_prior_s=previous, screen_prior_age_s=age, screen_bracketing_s=around,
            screen_update_group=refresh_group(previous, age), active_link_interval_s=link_seconds,
            screen_max_hz=key_screen[1] if key_screen else None,
            key_screen_last_update_s=key_screen[2] if key_screen else None,
            lag_overlap=bool(overlaps),
            max_lateness_ms=max(((actual - ready) * 1000 for ready, actual in overlaps), default=None),
        ))
    return rows, dict(pinned=pinned, pauses=len(pauses), active_ticks=len(link), unsupported=unsupported,
                      stalls_over_2ms=len(stalls), stalls_overlapping_misses=len(missed_stalls),
                      sck_status=Counter(r["status"] for r in sck_rows),
                      render_links=render_links, measured_seconds=summary["seconds"])


def utc():
    return datetime.now(timezone.utc).isoformat()


def wait_spotlight(root):
    """Check the sum in ps units (100% is one core) every 15 seconds."""
    start = time.monotonic()
    below_since = None
    with (root / "spotlight.jsonl").open("a", buffering=1) as log:
        while True:
            cpu = dict.fromkeys(("mds", "mds_stores", "mds_worker"), 0.0)
            process_list = subprocess.check_output(["/bin/ps", "-Ao", "pcpu=,comm="], text=True)
            for line in process_list.splitlines():
                amount, path = line.strip().split(maxsplit=1)
                name = Path(path).name
                if name in cpu:
                    cpu[name] += float(amount)
            now = time.monotonic()
            total = sum(cpu.values())
            if total >= QUIET_LIMIT:
                below_since = None
            elif below_since is None:
                below_since = now
            row = dict(utc=utc(), seconds=now-start, cpu=cpu, total=total,
                       quiet_seconds=now-below_since if below_since is not None else 0)
            log.write(json.dumps(row) + "\n")
            if row["quiet_seconds"] >= 120 or row["seconds"] >= 1200:
                row["passed"] = row["quiet_seconds"] >= 120
                (root / "spotlight-gate.json").write_text(json.dumps(row) + "\n")
                return row
            time.sleep(min(15, 1200 - row["seconds"]))


def run_matrix(args):
    args.output.mkdir(parents=True, exist_ok=True)
    gate_path = args.output / "spotlight-gate.json"
    gate = json.loads(gate_path.read_text()) if gate_path.exists() else wait_spotlight(args.output)
    if not gate["passed"]:
        raise SystemExit("Spotlight did not stay below 5% for 120 seconds. Matrix was not started.")
    groups = args.output / "pairs"
    groups.mkdir(exist_ok=True)
    with (args.output / "matrix.log").open("a", buffering=1) as log:
        for number in range(1, 6):
            for app in APPS:
                # One control in the same block serves both contrasts. In the
                # reversed blocks it follows both experimental captures.
                modes = MODES if number % 2 else MODES[::-1]
                missing = [mode for mode in modes if not (groups / f"{app}-{number}-{mode}" / "summary.json").exists()]
                if not missing:
                    continue
                if time.time() + len(missing) * 75 >= args.deadline:
                    raise SystemExit("Ticket cutoff reached before the next matched block.")
                load_dir = groups / f"{app}-{number}-load"
                load_dir.mkdir(exist_ok=True)
                quiet_deadline = min(time.monotonic() + 600,
                                     time.monotonic() + args.deadline - time.time() - len(missing) * 75)
                for sample in range(100):
                    snapshot = collect_load(str(load_dir / f"{time.time_ns()}-{sample}.txt"), [], None)
                    background = snapshot["background_cpu_percent"]
                    if ((number == 1 and app == APPS[0]) or background <= QUIET_LIMIT
                            or time.monotonic() >= quiet_deadline):
                        break
                    time.sleep(max(0, min(60, quiet_deadline - time.monotonic())))
                loaded = background > QUIET_LIMIT
                (load_dir / "chosen.json").write_text(json.dumps(dict(snapshot, loaded=loaded), indent=2) + "\n")
                log.write(f"{utc()} {app} block {number} {'loaded' if loaded else 'quiet'} {background:.2f}%\n")
                for mode in missing:
                    out = groups / f"{app}-{number}-{mode}"
                    if out.exists():
                        previous = args.output / "incomplete"
                        previous.mkdir(exist_ok=True)
                        shutil.move(out, previous / f"{out.name}-{time.time_ns()}")
                    flags = dict(os.environ, GPUI_FAST_NO_DISPLAY_SYNC="1", GPUI_FAST_LATE_DRAWABLE="1",
                                 RUSTUI_G5_TRIM_EDITOR="1", RUSTUI_MAIN_LAG="1",
                                 RUSTUI_G5_SCK_CADENCE="1", RUSTUI_G5_KEEP_WARM=mode)
                    with out.with_suffix(".driver").open("w") as evidence:
                        subprocess.run([sys.executable, "scripts/run_g5_budget.py", "--app", app,
                                        "--scene", "typing", "--hz", "10", "--launch", "direct",
                                        "--external-observer", "--seconds", "60", "--scheduler", "1",
                                        "--input-scoped", "1", "--switches", "1111", "--passive", "0",
                                        "--pixels", "1", "--output", str(out)],
                                       env=flags, stdout=evidence, stderr=subprocess.STDOUT, check=True)
                    inspect_capture(out, app, mode)
                    log.write(f"{utc()} complete {app} {number} {mode}\n")


def summarize(root):
    groups = root / "pairs"
    report = {}
    rows = []
    for app in APPS:
        frozen = [json.loads(p.read_text())["frozen"] for p in groups.glob(f"{app}-*/method.json")
                  if (p.parent / "summary.json").exists()]
        assert not frozen or all(artifacts == frozen[0] for artifacts in frozen), f"{app} changed build"
        report[app] = {}
        for mode in MODES:
            captures = {}
            changes = []
            per_attempt_changes = []
            for i in range(1, 6):
                candidate = groups / f"{app}-{i}-{mode}"
                off = groups / f"{app}-{i}-off"
                if not (candidate / "summary.json").exists() or not (off / "summary.json").exists():
                    continue
                data, details = inspect_capture(candidate, app, mode)
                chosen = json.loads((groups / f"{app}-{i}-load" / "chosen.json").read_text())
                candidate_cpu = json.loads((candidate / "summary.json").read_text())
                off_cpu = json.loads((off / "summary.json").read_text())
                raw_change = 100 * (candidate_cpu["cpu_s_per_s"] / off_cpu["cpu_s_per_s"] - 1)
                if app != "appkit":
                    per_attempt_changes.append(
                        100 * (candidate_cpu["cpu_s_per_attempt"] / off_cpu["cpu_s_per_attempt"] - 1))
                changes.append(raw_change)
                captures[i] = (data, dict(details, load=chosen["background_cpu_percent"], loaded=chosen["loaded"]))
            pooled = [row for capture, _ in captures.values() for row in capture]
            counts = Counter(row["classification"] for row in pooled)
            denominator = counts["hit"] + counts["miss"]
            primary = changes if app == "appkit" else per_attempt_changes
            rows.extend(pooled)
            report[app][mode] = dict(
                completed=len(captures), counts=counts,
                miss_percent=100 * counts["miss"] / denominator if denominator else None,
                handler_sck_ms=distribution([r["handler_sck_ms"] for r in pooled if r["handler_sck_ms"] is not None]),
                p_ms=distribution([r["P_ms"] for r in pooled if r["P_ms"] is not None]),
                cpu_s_per_s_paired_percent=changes if mode != "off" else [],
                cpu_per_attempt_paired_percent=per_attempt_changes if mode != "off" else [],
                cpu_verdict=p_verdict(primary) if mode != "off" and len(primary) == 5 else None,
                refresh_by_class={group: dict(
                    prior_screen_groups=Counter(r["screen_update_group"] for r in pooled if r["classification"] == group),
                    sck_interval_ms=distribution([r["sck_bracketing_s"] * 1000 for r in pooled if r["classification"] == group and r["sck_bracketing_s"] is not None]),
                    active_link_ms=distribution([r["active_link_interval_s"] * 1000 for r in pooled if r["classification"] == group and r["active_link_interval_s"] is not None]),
                    lag_overlap=sum(r["lag_overlap"] for r in pooled if r["classification"] == group),
                ) for group in ("hit", "miss", "no_gpu_clock")},
                captures={i: info for i, (_, info) in captures.items()},
            )
    if rows:
        with (root / "per-key.csv").open("w", newline="") as out:
            writer = csv.DictWriter(out, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)
    (root / "analysis.json").write_text(json.dumps(report, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("run", "analyze"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--deadline", type=float, help="Hard latest end time as Unix seconds")
    args = parser.parse_args()
    if args.action == "run":
        if args.deadline is None:
            parser.error("--deadline is required for the timed matrix")
        run_matrix(args)
    else:
        summarize(args.output)


if __name__ == "__main__":
    main()
