#!/usr/bin/env python3
"""Run native G5-F1 functional checks or the matched capture matrix."""
import argparse
import hashlib
import json
import statistics
import subprocess
import sys
from pathlib import Path

from g5_latency import p_verdict

ROOT = Path(__file__).resolve().parents[1]


def run_capture(output, *, app="lab", scene="typing", hz=10, switches="1111",
                scheduler=1, seconds=5, trace=False):
    command = [
        sys.executable, str(ROOT / "scripts/run_g5_budget.py"),
        "--app", app, "--scene", scene, "--hz", str(hz), "--switches", switches,
        "--scheduler", str(scheduler), "--seconds", str(seconds),
        "--pixels", "0", "--passive", "0", "--output", str(output),
    ]
    if seconds == 5:
        command += ["--timeout", "10" if scene == "typing" else "25"]
    if trace:
        command += ["--trace-wakes"]
    with output.with_suffix(".txt").open("w") as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    print(output.name, "exit", result.returncode, flush=True)
    if result.returncode:
        return None
    return json.loads((output / "summary.json").read_text())


def assert_correct(capture):
    assert capture is not None, "native workload failed"
    assert capture["out_of_order"] == 0, "out-of-order presentation"
    assert capture["lost_edits"] in (0, None), "lost edit"
    assert capture["watchdog_fallbacks"] == 0, "watchdog fired during normal workload"
    assert not capture["versions_without_later_presentation"], "final delivered input not shown"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("reproduce", "proof", "pairs"))
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    switches = ("0000", "1111", "1000", "0100", "0010", "0001", "1100", "1010", "1001")
    if args.mode == "reproduce":
        for arm in switches:
            run_capture(output / arm, switches=arm, scheduler=0, trace=True)
        return
    if args.mode == "proof":
        for arm in switches:
            assert_correct(run_capture(output / arm, switches=arm))
        for app, scene in (("bare", "typing"), ("lab", "hover"), ("lab", "drag")):
            assert_correct(run_capture(output / f"{app}-{scene}", app=app, scene=scene))
        return

    refs = ROOT / "target/headstart/g5-1"
    executables = (
        Path.home() / ".cache/rustui-target/release/rustui-lab",
        refs / "libG5Native.dylib", refs / "libG5Pixels.dylib", refs / "libG5Refresh.dylib",
    )
    def fingerprint():
        return {str(path): hashlib.file_digest(path.open("rb"), "sha256").hexdigest()
                for path in executables}

    frozen = fingerprint()
    (output / "frozen.json").write_text(json.dumps(frozen, indent=2) + "\n")
    for scene, hz in (("typing", 10), ("typing", 30), ("hover", 10), ("drag", 10)):
        results = []
        for pair in range(1, 6):
            captures = {}
            for arm in (("off", "on") if pair % 2 else ("on", "off")):
                stem = output / f"{scene}-{hz}-{pair}-{arm}"
                captures[arm] = run_capture(
                    stem, scene=scene, hz=hz, switches="1111" if arm == "on" else "0000",
                    seconds=60,
                )
                assert fingerprint() == frozen, "build changed during the matrix"
                assert_correct(captures[arm])
            results.append(captures)
        changes = [100 * (pair["on"]["cpu_s_per_s"] / pair["off"]["cpu_s_per_s"] - 1)
                   for pair in results]
        table = {}
        for arm in ("off", "on"):
            table[arm] = {
                field: {quantile: statistics.median(
                    pair[arm]["stats"][field][quantile] for pair in results)
                    for quantile in ("median", "p95")}
                for field in ("hardware_presented_total", "presented_total")
            }
            table[arm]["cpu_s_per_s"] = statistics.median(pair[arm]["cpu_s_per_s"] for pair in results)
        summary = dict(changes=changes, verdict=p_verdict(changes), arms=table)
        (output / f"{scene}-{hz}.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
