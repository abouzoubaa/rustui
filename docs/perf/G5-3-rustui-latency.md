# G5-3b — RustUI input-to-screen latency

PR #173 remains a draft until every requested correctness, CPU and latency target is established. This ticket uses the existing `codex/g5-3-rustui-latency` branch from g5-integration. The previous G5-3 results remain in the PR history and the local G5-3 report; they are not relabelled as new measurements.

## Reserve arm retired

The owner dropped `GPUI_FAST_DRAWABLE_RESERVE_TWO`: cadence fell to 76–78 visible versions/s and CPU regressed. A 5.602 ms acquisition occurred with only 1/3 slots occupied. Those observations do not support pool saturation as the explanation for the waits. The reserve patch, its stack entries and its launcher arm are removed. All acquisitions over 2 ms remain known; this ticket adds no acquisition workaround.

## Implementation and method

Every new behavior stays off by default. `RUSTUI_DRAW_FIRST=1` prioritizes a completed Kit edit ahead of queued binding/owner/change-listener effects, after the editing entity has been returned to GPUI. It never draws while borrowing that entity. The CodeEditor continues painting the local Kit text while the owner is stale, including a committed IME value. Validation that rejects an edit remains before mutation; queued validation/change listeners run after the prioritized draw.

The same switch moves tree-sitter parsing, injections and highlight capture matching to GPUI's background executor using an immutable text/tree/query snapshot. Only a matching latest-text result can apply. Painting reads an indexed immutable capture cache and resolves current theme styles on the main thread. Pending edits translate visible cached spans without rewriting all document captures, preserving existing text's span positions until the new colours arrive. The first frame can use the preceding span colours. The worker currently matches the full document per edit: its total CPU cost is included in the paired verdict, and the async worker does not reuse PX-3's negative-injection proof.

`GPUI_FAST_IMMEDIATE_COALESCE=1` limits submissions separately per renderer/surface to one platform refresh interval and skips an already-submitted scene generation. Dirty changes declined for the current interval are retried on GPUI's executor; order and source/capacity guards remain per surface. Scene generations distinguish a genuinely new colour update from a keepalive presentation of the same scene. `GPUI_FAST_IMMEDIATE_ANY_CHANGE=1` routes idle timer/append/async invalidation through the same immediate path. Live resize retains its synchronous full-render route.

The off arm uses the normal display-link path (G5-1 immediate presentation and all three new switches off). The on arm enables G5-1 plus all three new switches. This compares total CPU against the original non-immediate baseline, so an improvement over an already-regressed immediate arm cannot conceal a regression. PX-3 is on in both typing arms. All six workloads use one pinned Rust 1.98.0 release build, damage off, buffered logging and identical diagnostics, five matched 60-second pairs after normal warmup, alternating off→on / on→off / off→on / on→off / off→on. No build, profiling or heavy analysis overlaps timing. Corrected background CPU and the top five processes are saved at each capture; a failed quiet check is labelled loaded rather than discarded under the owner's bypass.

The deciding CPU metric for G5-3b is total process CPU seconds per measured wall second. CPU per attempt remains diagnostic because coalescing changes the attempt count. Each capture takes getrusage user+sys snapshots after warmup. Input-to-screen uses MTLDrawable.presentedTime; the passive display-link observer supplies the actual observed refresh cadence. Missed-refresh figures retain the disclosed two-refresh G5-1 control floor. No new control floor or quiet-machine confirmation is claimed.

Workloads: typing 10/s and 30/s (10,000-line Rust editor, PX-3 on), hover, drag, F0 Collection, log. A Kit document-revision witness additionally checks every typing edit, so a matching net final string cannot conceal a lost insert/delete pair. Scroll-offset witnesses discount Collection presentations with no changed scroll position; unrendered intermediate offsets are not fabricated.

## Keystroke trace

The first diagnostic trace corrects the starting hypothesis: before the first CPU draw, Kit already held the new local text, while the RustUI owner still held the previous value. The first draw did not acquire/submit a drawable because a display link was reattached during that draw. Binding propagation followed; a later display-link draw submitted. This is a frame-source handoff delay, not evidence that the first draw read the owner's old text. The final paint witness records Kit's actual text length and document revision, independently of the input-version label.

Exact baseline, candidate and bare-example timestamps are filled from the final frozen-build traces below. Short diagnostic captures receive no P verdict.

## Validation

`scripts/check.sh`: exit 0, 1,876 total across seven groups (1,869 passed, 7 ignored). Ten editor tests also pass with the opt-in enabled, including drawing before binding effects, IME commit and undo. Five worker/cache colour oracles pass. The Metal copy/clear/retained/Skip/magenta Partial proofs, native hover pixel check and F0 Collection capture check pass. Both stacks pass: 84 ordered patches plus one alternative, 150 exact postimages.

The verifier/measurement focused suite passes 36 tests, including candidate tampering, default rejection, Cargo anchoring and exact restoration with created files absent. Explicit Kit candidate checking passes. The integration source flow runs 728 tests and fails with two errors in unchanged `test_cargo_headstart` cases: `test_explicit_rustc_is_preserved_and_default_stays_headstart` and `test_tool_rustdoc_restores_library_path_and_preserves_caller_override`; both report the unchanged launcher exiting 1. This ticket does not modify that launcher or its tests. The approved-only release source command rejects the applied candidate as required. Source CI is not green, and no merge is made.

There are 56 retained 60-second captures: five matched pairs for each of the first five workloads, three for log. Runtime hashes before/after are identical; windows range from 60.001409 to 60.105747 seconds. All 247,164 raw GPU records are Direct. Recorded state/version checks show no lost edits or capacity violations. Strictly later presentedTime comparisons show zero backwards versions; one equal-time slot remains explicitly ambiguous. Two measured acquisitions exceed 2 ms and are listed below. Corrected starting background load ranges from 5.98% to 27.95%; each capture retains its own quiet/loaded label.

Acceptance fails. Both typing medians improve to approximately 23.08 ms, but every complete workload has a total-process CPU REGRESSION. Typing 10/s submits 8,570 GPU frames for 3,000 edits, so the no-extra-frame target is not met. Hover and drag also miss the 30 ms median target. Log has only three pairs and receives no P verdict; its remaining two pairs were not started within the timebox. PR #173 stays draft; all new behavior remains off by default.

## Approved-only Kit verification

The default `python3 scripts/apply_gpui_kit_09_stack.py --check` accepts approved bytes only. The integration-only path is explicit:

```sh
python3 scripts/apply_gpui_kit_09_stack.py --check --candidate-stack patches/candidate-fast-latency/latency-stack.json
python3 scripts/check_source_candidates.py --candidate-stack patches/candidate-fast-latency/latency-stack.json
```

A pinned source/Cargo-checksum catalogue from the approved v0.5.0 commit covers every file in the two affected Kit crates (Base and Component) and their owned mirrors. All existing approved-stack checks remain in place. Existing approved patch hashes remain authoritative where historical Cargo metadata predates an approved patch; previously unlisted unchanged files require their pre-candidate Cargo hash. No candidate post-image is added to the approved catalogue. Verification reconstructs the approved bytes in a temporary fixture, replays the hashed candidate chain, and reverses it again, including absence of created files. The explicit integration entrypoint reuses every current source-mode check verbatim, supplying the candidate option only to the Kit check. The release verifier and CI are not changed to opt in automatically. Consequently approved-only source verification must reject this applied, unpromoted candidate checkout; its rejection is reported rather than bypassed.

## Equal-time presentation callbacks

During typing 30/s pair 3 off, drawable callbacks for versions 48 and 49 had exactly the same `presentedTime` (43453.11828391668 seconds). Converting each callback clock separately introduced an 83 ns ordering difference. The recorder's initial assertion stopped on that artificial inversion. The saved capture was retained and resumed without a runtime change or recapture. Order now compares strictly later presentedTime values; equal-time callbacks are reported as ambiguous. Visible-version throughput counts one presentation slot per refresh, and excludes overlapping possible-version sets that cannot prove a new visible change. It is a conservative rate, not a pixel claim about which tied drawable won. All saved captures are analyzed with this same corrected method.

## G5-3b matched captures

Quantiles below are medians of the five capture quantiles; maxima are maxima across captures. Input delays use MTLDrawable.presentedTime. Each capture has its own measured background load below. No unavailable callback is counted as physical proof that pixels were absent.

| workload | arm | captures / 5 | input→display median / p95 / max ms | visible versions/s | unmatched / rendered | order violations | lost native edits | >2 ms measured acquisitions | CPU s/s | CPU ms/attempt | draw+present ms/attempt | frames >8.3 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 5 | 49.254/58.105/70.679 | 9.146 | 5/2741 | 0 | 0 | 0 | 0.111 | 12.151 | 2.140 | 0 |
| typing-10 | on | 5 | 23.080/27.476/35.482 | 9.972 | 0/8570 | 0 | 0 | 0 | 0.328 | 3.412 | 0.564 | 0 |
| typing-30 | off | 5 | 45.769/49.914/68.632 | 26.201 | 6/7902 | 0 | 0 | 0 | 0.201 | 7.678 | 1.807 | 0 |
| typing-30 | on | 5 | 23.084/33.249/35.541 | 29.965 | 0/25193 | 0 | 0 | 0 | 0.746 | 2.549 | 0.404 | 0 |
| hover | off | 5 | 45.768/49.796/54.212 | 92.830 | 2867/35981 | 0 | 0 | 0 | 0.302 | 2.520 | 1.512 | 1 |
| hover | on | 5 | 30.143/36.379/43.312 | 101.437 | 12/30451 | 0 | 0 | 0 | 0.406 | 1.109 | 0.897 | 0 |
| drag | off | 5 | 45.773/49.690/57.704 | 93.411 | 2621/35963 | 0 | 0 | 0 | 0.212 | 1.771 | 0.456 | 1 |
| drag | on | 5 | 30.826/39.663/50.019 | 100.231 | 9/30095 | 0 | 0 | 2 | 0.240 | 0.729 | 0.300 | 1 |
| acceptance | off | 5 | 45.782/49.447/63.067 | 99.184 | 48/35951 | 0 | 0 | 0 | 0.420 | 3.502 | 3.015 | 1374 |
| acceptance | on | 5 | 27.430/34.867/54.099 | 100.000 | 9/30043 | 0 | 0 | 0 | 0.613 | 2.356 | 2.170 | 271 |
| log | off | 3 | 45.794/49.824/57.724 | 9.566 | 3/1726 | 0 | 0 | 0 | 0.069 | 7.200 | 2.996 | 1 |
| log | on | 3 | 24.623/28.978/32.728 | 9.566 | 3/1726 | 0 | 0 | 0 | 0.066 | 6.915 | 2.973 | 1 |

## Paired CPU gate

Deciding metric: getrusage user+sys seconds divided by measured wall seconds. The five pairs alternate off→on, on→off, off→on, on→off, off→on. Negative is less CPU. GAIN: all five ≤−3%; REGRESSION: all five >+3%; NEUTRAL: all five within inclusive ±3%; otherwise INCONCLUSIVE. Incomplete sets have no verdict. CPU per attempt is diagnostic only for this ticket.

| workload | five total-process CPU/s paired changes % | median / min / max % | verdict | five CPU/attempt changes %, diagnostic |
| --- | --- | --- | --- | --- |
| typing-10 | 210.338, 179.364, 173.201, 198.567, 192.393 | 192.393/173.201/210.338 | REGRESSION | -70.300, -73.595, -74.271, -71.532, -72.082 |
| typing-30 | 247.669, 337.505, 293.833, 273.192, 255.529 | 273.192/247.669/337.505 | REGRESSION | -68.838, -59.564, -64.361, -66.964, -68.064 |
| hover | 30.872, 34.138, 33.753, 37.202, 36.968 | 34.138/30.872/37.202 | REGRESSION | -57.335, -55.994, -55.992, -56.017, -54.923 |
| drag | 12.901, 15.004, 14.823, 9.245, 8.396 | 12.901/8.396/15.004 | REGRESSION | -58.605, -57.941, -58.598, -60.284, -61.730 |
| acceptance | 51.875, 46.649, 46.512, 42.958, 45.825 | 46.512/42.958/51.875 | REGRESSION | -33.815, -32.902, -32.595, -31.878, -32.119 |
| log | -3.375, -12.053, -3.908 | -3.908/-12.053/-3.375 | INCOMPLETE | -3.368, -12.049, -3.914 |

## Frame and refresh diagnostics

Throughput is a conservative count of distinct presentedTime slots with provably changed version sets. Multiple callbacks with different versions at exactly the same display time are ambiguous, not an inferred ordering.

The same input version can legitimately acquire later colours. Scene generations distinguish those updates from presenting an already-submitted unchanged scene. The single-refresh rule is per renderer/surface, using the platform refresh interval; it does not claim to read WindowServer private queue state.

| workload | arm | CPU draws / GPU frames / inputs | GPU→display median ms | missed refreshes median / p95 | gaps median / p95 / max ms | same-scene submissions skipped | next-interval deferrals | equal-time ambiguous slots |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | off | 2995/2741/3000 | 38.309 | 2.000/2.000 | 108.333/116.668/133.332 | 0 | 0 | 0 |
| typing-10 | on | 28980/8570/3000 | 20.894 | 0.000/0.000 | 16.667/79.166/83.333 | 0 | 20270 | 0 |
| typing-30 | off | 8995/7902/9000 | 38.439 | 2.000/2.000 | 41.666/41.667/66.666 | 0 | 0 | 1 |
| typing-30 | on | 88210/25193/9000 | 21.269 | 0.000/0.000 | 8.333/16.667/25.000 | 0 | 62423 | 0 |
| hover | off | 29681/35981/36000 | 39.001 | 2.000/2.000 | 8.333/16.666/25.000 | 0 | 0 | 0 |
| hover | on | 110196/30451/36000 | 21.417 | 0.000/1.000 | 8.333/16.667/25.000 | 0 | 79745 | 0 |
| drag | off | 29633/35963/36000 | 39.695 | 2.000/2.000 | 8.333/16.666/25.000 | 0 | 0 | 0 |
| drag | on | 99392/30095/36000 | 22.063 | 0.000/1.000 | 8.333/16.667/25.001 | 0 | 69297 | 0 |
| acceptance | off | 30034/35951/36000 | 38.317 | 2.000/2.000 | 8.333/8.333/25.000 | 0 | 0 | 0 |
| acceptance | on | 78377/30043/36000 | 20.657 | 0.000/0.000 | 8.333/16.667/41.666 | 0 | 48334 | 0 |
| log | off | 1726/1726/1800 | 36.772 | 2.000/2.000 | 104.166/108.334/116.667 | 0 | 0 | 0 |
| log | on | 1726/1726/1800 | 20.070 | 0.000/0.000 | 104.166/108.333/116.666 | 0 | 0 | 0 |

## Observed-refresh missed slots

These labels classify observable trace states, with the inherited two-refresh control floor. “WindowServer late” means GPU-ready and handed off but not observed in the eligible slot; it does not establish compositor causality. Observer gaps are disclosed rather than filled in.

| workload | arm | slots | no new input | frame not ready | no drawable free | presented but replaced | WindowServer late | observer gaps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hover | off | 35999 | 5 | 15 | 0 | 3222 | 4905 | 2 |
| hover | on | 36001 | 5 | 4005 | 0 | 16 | 1551 | 1 |
| drag | off | 36002 | 5 | 16 | 0 | 2979 | 4965 | 0 |
| drag | on | 35999 | 6 | 2623 | 0 | 14 | 3288 | 3 |

## Per-capture process CPU and load

Each loaded row means **measured on a loaded machine (X% background)**; X is the start→end background percentage shown. Corrected background excludes Codex, its agents/shells and the measured app. Raw top-five process lists remain with each local capture.

| workload | pair | arm | attempts | wall s | user / sys / total CPU s | CPU s/s | CPU ms/attempt | actual refresh Hz | background start→end % | label |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| typing-10 | 1 | off | 552 | 60.059 | 5.235/1.092/6.327 | 0.105 | 11.462 | 120.000 | 5.980→7.100 | quiet |
| typing-10 | 1 | on | 5768 | 60.060 | 18.523/1.113/19.636 | 0.327 | 3.404 | 120.000 | 10.190→16.800 | loaded |
| typing-10 | 2 | on | 5768 | 60.061 | 18.574/1.106/19.680 | 0.328 | 3.412 | 120.000 | 11.040→18.210 | loaded |
| typing-10 | 2 | off | 545 | 60.042 | 5.903/1.139/7.042 | 0.117 | 12.922 | 120.000 | 17.850→14.490 | loaded |
| typing-10 | 3 | off | 543 | 60.066 | 6.090/1.111/7.201 | 0.120 | 13.261 | 120.000 | 11.270→16.380 | loaded |
| typing-10 | 3 | on | 5760 | 60.005 | 18.600/1.053/19.653 | 0.328 | 3.412 | 120.000 | 7.230→13.480 | loaded |
| typing-10 | 4 | on | 5776 | 60.106 | 18.767/1.162/19.929 | 0.332 | 3.450 | 120.000 | 19.680→20.960 | loaded |
| typing-10 | 4 | off | 550 | 60.025 | 5.431/1.235/6.666 | 0.111 | 12.120 | 120.000 | 18.030→28.510 | loaded |
| typing-10 | 5 | off | 551 | 60.095 | 5.450/1.245/6.695 | 0.111 | 12.151 | 120.000 | 27.950→20.220 | loaded |
| typing-10 | 5 | on | 5768 | 60.066 | 18.520/1.048/19.568 | 0.326 | 3.392 | 120.000 | 16.440→24.720 | loaded |
| typing-30 | 1 | off | 1560 | 60.028 | 10.640/2.238/12.878 | 0.215 | 8.255 | 120.000 | 25.080→14.210 | loaded |
| typing-30 | 1 | on | 17400 | 60.011 | 42.493/2.266/44.759 | 0.746 | 2.572 | 120.000 | 16.670→15.620 | loaded |
| typing-30 | 2 | on | 17366 | 60.005 | 42.719/1.817/44.536 | 0.742 | 2.565 | 120.000 | 9.060→19.890 | loaded |
| typing-30 | 2 | off | 1606 | 60.040 | 8.442/1.744/10.186 | 0.170 | 6.342 | 120.000 | 19.170→20.540 | loaded |
| typing-30 | 3 | off | 1592 | 60.035 | 9.399/1.980/11.380 | 0.190 | 7.148 | 120.000 | 18.330→22.630 | loaded |
| typing-30 | 3 | on | 17593 | 60.036 | 42.853/1.965/44.818 | 0.747 | 2.547 | 120.000 | 16.520→8.610 | loaded |
| typing-30 | 4 | on | 17727 | 60.017 | 43.002/1.965/44.967 | 0.749 | 2.537 | 120.000 | 13.950→24.250 | loaded |
| typing-30 | 4 | off | 1569 | 60.007 | 10.008/2.040/12.047 | 0.201 | 7.678 | 120.000 | 16.030→25.320 | loaded |
| typing-30 | 5 | off | 1575 | 60.036 | 10.421/2.151/12.572 | 0.209 | 7.982 | 120.000 | 21.930→20.390 | loaded |
| typing-30 | 5 | on | 17530 | 60.023 | 42.747/1.939/44.686 | 0.744 | 2.549 | 120.000 | 15.220→17.070 | loaded |
| hover | 1 | off | 7196 | 60.002 | 13.382/5.254/18.636 | 0.311 | 2.590 | 120.000 | 17.500→16.730 | loaded |
| hover | 1 | on | 22073 | 60.002 | 21.419/2.970/24.389 | 0.406 | 1.105 | 120.000 | 6.440→15.730 | loaded |
| hover | 2 | on | 21948 | 60.008 | 21.608/2.734/24.342 | 0.406 | 1.109 | 120.000 | 14.750→21.790 | loaded |
| hover | 2 | off | 7200 | 60.005 | 13.108/5.038/18.146 | 0.302 | 2.520 | 120.000 | 13.480→7.300 | loaded |
| hover | 3 | off | 7196 | 60.001 | 13.116/5.043/18.159 | 0.303 | 2.523 | 120.000 | 15.130→16.260 | loaded |
| hover | 3 | on | 21873 | 60.008 | 21.305/2.985/24.290 | 0.405 | 1.111 | 120.000 | 14.450→15.400 | loaded |
| hover | 4 | on | 22443 | 60.002 | 21.473/2.948/24.422 | 0.407 | 1.088 | 120.000 | 13.110→16.880 | loaded |
| hover | 4 | off | 7195 | 60.005 | 12.954/4.847/17.801 | 0.297 | 2.474 | 120.000 | 9.200→23.860 | loaded |
| hover | 5 | off | 7194 | 60.009 | 13.017/4.791/17.808 | 0.297 | 2.475 | 120.000 | 19.540→16.030 | loaded |
| hover | 5 | on | 21859 | 60.008 | 21.519/2.872/24.391 | 0.406 | 1.116 | 120.000 | 14.460→18.640 | loaded |
| drag | 1 | off | 7192 | 60.008 | 6.245/6.491/12.736 | 0.212 | 1.771 | 120.000 | 14.980→17.320 | loaded |
| drag | 1 | on | 19614 | 60.003 | 8.514/5.864/14.377 | 0.240 | 0.733 | 120.000 | 14.410→14.160 | loaded |
| drag | 2 | on | 19651 | 60.009 | 8.462/5.859/14.321 | 0.239 | 0.729 | 120.000 | 14.230→16.440 | loaded |
| drag | 2 | off | 7186 | 60.003 | 6.186/6.265/12.451 | 0.208 | 1.733 | 120.000 | 18.450→15.280 | loaded |
| drag | 3 | off | 7197 | 60.008 | 6.260/6.397/12.657 | 0.211 | 1.759 | 120.000 | 13.380→20.110 | loaded |
| drag | 3 | on | 19961 | 60.011 | 8.656/5.878/14.534 | 0.242 | 0.728 | 120.000 | 18.160→16.420 | loaded |
| drag | 4 | on | 19791 | 60.003 | 8.669/5.865/14.533 | 0.242 | 0.734 | 120.000 | 22.650→20.720 | loaded |
| drag | 4 | off | 7195 | 60.003 | 6.546/6.758/13.304 | 0.222 | 1.849 | 120.000 | 13.570→15.060 | loaded |
| drag | 5 | off | 7193 | 60.004 | 6.517/6.670/13.188 | 0.220 | 1.833 | 120.000 | 14.440→23.060 | loaded |
| drag | 5 | on | 20375 | 60.008 | 8.526/5.770/14.296 | 0.238 | 0.702 | 120.000 | 17.360→26.220 | loaded |
| acceptance | 1 | off | 7191 | 60.010 | 9.759/15.503/25.262 | 0.421 | 3.513 | 120.000 | 17.180→17.310 | loaded |
| acceptance | 1 | on | 16499 | 60.002 | 13.894/24.468/38.362 | 0.639 | 2.325 | 120.000 | 13.250→14.360 | loaded |
| acceptance | 2 | on | 15706 | 60.008 | 13.763/23.244/37.007 | 0.617 | 2.356 | 120.000 | 14.060→16.110 | loaded |
| acceptance | 2 | off | 7186 | 60.007 | 9.866/15.368/25.234 | 0.421 | 3.512 | 120.000 | 21.660→15.360 | loaded |
| acceptance | 3 | off | 7194 | 60.010 | 9.688/15.420/25.108 | 0.418 | 3.490 | 120.000 | 18.050→20.190 | loaded |
| acceptance | 3 | on | 15637 | 60.010 | 13.904/22.882/36.786 | 0.613 | 2.353 | 120.000 | 21.570→21.650 | loaded |
| acceptance | 4 | on | 15093 | 60.003 | 13.056/22.949/36.005 | 0.600 | 2.386 | 120.000 | 18.700→23.660 | loaded |
| acceptance | 4 | off | 7192 | 60.002 | 9.910/15.275/25.185 | 0.420 | 3.502 | 120.000 | 20.360→23.570 | loaded |
| acceptance | 5 | off | 7188 | 60.008 | 10.251/14.754/25.005 | 0.417 | 3.479 | 120.000 | 24.240→21.690 | loaded |
| acceptance | 5 | on | 15442 | 60.009 | 13.526/22.939/36.465 | 0.608 | 2.361 | 120.000 | 20.470→28.030 | loaded |
| log | 1 | off | 575 | 60.012 | 3.037/1.104/4.140 | 0.069 | 7.200 | 120.000 | 22.740→19.510 | loaded |
| log | 1 | on | 575 | 60.017 | 2.996/1.004/4.001 | 0.067 | 6.958 | 120.000 | 19.660→19.640 | loaded |
| log | 2 | on | 575 | 60.003 | 2.982/0.995/3.976 | 0.066 | 6.915 | 120.000 | 20.760→15.600 | loaded |
| log | 2 | off | 576 | 60.105 | 3.430/1.099/4.529 | 0.075 | 7.863 | 120.000 | 13.090→16.520 | loaded |
| log | 3 | off | 575 | 60.004 | 3.105/1.007/4.112 | 0.069 | 7.152 | 120.000 | 13.950→15.500 | loaded |
| log | 3 | on | 576 | 60.104 | 3.029/0.929/3.958 | 0.066 | 6.872 | 120.000 | 13.630→18.270 | loaded |

## Known acquisitions over 2 ms

All phases are listed. No acquisition workaround is adopted from these numbers. Each entry gives phase, path, duration, pre-call live/pool, attached state and milliseconds since display-link detach.

- typing-10 pair 2 off: warmup, non-immediate, 4.339375 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 1 off: warmup, non-immediate, 4.045583 ms, 0/3, attached=False, detach age=N/A ms.

- hover pair 3 off: warmup, non-immediate, 4.582125 ms, 0/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.039416 ms, 1/3, attached=False, detach age=N/A ms.

- drag pair 2 on: measured, immediate, 2.634208 ms, 2/3, attached=False, detach age=N/A ms.

- acceptance pair 5 off: warmup, non-immediate, 4.066167 ms, 0/3, attached=False, detach age=N/A ms.

- acceptance pair 5 on: warmup, non-immediate, 2.059958 ms, 0/3, attached=False, detach age=N/A ms.

## Single-keystroke timestamps

Diagnostic traces only; these short recordings have no P verdict. All values are milliseconds after the first handled key, measured with mach_absolute_time. Presented delay uses the drawable display time. “Not instrumented” is retained where an older trace did not have the final paint witness.

Bare G5-1 immediate: key tick 1017892846413, timebase 125/3; RustUI G5-1 immediate diagnostic: key tick 970769878496, timebase 125/3; RustUI new opt-in: key tick 1017392123632, timebase 125/3; RustUI new opt-in: submitted draw 38 painted local length 248395, document revision 2.

| stage | Bare G5-1 immediate | RustUI G5-1 immediate diagnostic | RustUI new opt-in |
| --- | --- | --- | --- |
| Kit local text applied | Not instrumented / not applicable | 0.303542 | 0.302125 |
| Highlighter update returns | Not instrumented / not applicable | 1.478875 | 0.309625 |
| Kit Change queued | Not instrumented / not applicable | 1.591375 | 0.493792 |
| Prioritized edit-frame callback | Not instrumented / not applicable | Not instrumented / not applicable | 0.503250 |
| RustUI Change listener / binding starts | Not instrumented / not applicable | 3.200542 | 3.920667 |
| Binding write / owner notify returns | Not instrumented / not applicable | 3.243583 | 3.981750 |
| Background colours applied | Not instrumented / not applicable | Not instrumented / not applicable | 40.000417 |
| Bare local text applied | 1.480042 | Not instrumented / not applicable | Not instrumented / not applicable |
| First CPU draw | 0.028250 | 1.640042 | 0.508167 |
| First GPU command committed | 4.039125 | 5.101667 | 2.876625 |
| GPU-completed callback | 6.856833 | 6.755125 | 4.191917 |
| Drawable display time | Not instrumented / not applicable | Not instrumented / not applicable | 20.743750 |

In the older RustUI immediate trace, the local Kit text had changed before the first CPU draw, while the owner value was old. A display-link recreate and an acquisition attempt with attached=true occurred during that draw; it produced no GPU commit. The later display-link draw submitted. The final opt-in trace separately witnesses the actual local text revision in the submitted paint. Binding/owner effects occur after the prioritized CPU draw attempt; if admission declines, this implementation does not hold those effects until a successful submission. That stronger guarantee remains unproven.

## Findings requiring another iteration

The per-surface cadence check runs inside MetalRenderer, after Window::draw has built the CPU scene. It therefore limits submissions while still paying for declined CPU draws. Scene generations also advance for binding/colour notifications of the same input version; this does not satisfy the requested semantic-version deduplication. The new colour worker generates captures for the full 10,000-line document per edit and always notifies on application. Those are avoidable new costs identified in the implementation; this cohort did not profile individual function costs. Lower CPU per attempt must not be called a gain when total CPU per second regresses. No exception or adoption is recommended.
