# SpringOpacity native scheduling probe

## Verification status

The final M60 aggregate passed in
`target/headstart/spring-opacity-verification-63179-1792336863069682000.json`;
the final verifier log is `target/headstart/milestone60-probe-verification-final.log`.
All four tests in `scripts/test_verify_spring_probe.py` passed.
Normal mode recorded root host lowerings `3, 13, 15, 26, 42, 42`: both passive
motion windows grew and the final 750 ms quiescence window stayed flat. Reduced
motion recorded `1, 1, 2, 2, 2, 2`: both passive windows and final quiescence
stayed flat while explicit updates caused the two lowerings.

The initial verifier attempt rejected emitted f32 values `0.15000000596046448`
against exact decimal `0.15`; its failure and generated artifacts remain in
`target/headstart/milestone60-probe-verification.log` and
`target/headstart/spring-opacity-scheduling-61162-1792336586290106000.json` /
`target/headstart/spring-opacity-scheduling-61210-1792336591264139000.json`.
The final verification accepts f32 values within tolerance.

This probe counts RustUI root host lowerings only. It does not measure GPUI
frames, presented alpha, or velocity continuity.

After adding the `--spring` mode and typed scopes, the four existing
`AnimatedOpacity` modes also passed the frame-limited regression: normal,
reduced motion, limited, and limited reduced motion all completed. Evidence:
`target/headstart/milestone60-existing-opacity-regression.log` and
`target/headstart/opacity-probe-verification-69223-1792337724122695000.json`.
This is a correctness regression result, not a frame-rate or performance claim.

`opacity_benchmark --spring` exercises the public `SpringOpacity` wrapper with one stable identity. It is a separate mode of the existing example. The four `AnimatedOpacity` modes and `scripts/verify_opacity_probe.py` keep their existing result format.

## Run

Build with the repository launcher, then run both spring modes through the verifier:

```sh
./scripts/cargo-headstart build -p rustui --example opacity_benchmark
python3 scripts/verify_spring_probe.py
```

To inspect an individual result, run `target/headstart/debug/examples/opacity_benchmark --spring` or add `--reduce-motion`. Each process prints the path of a unique `target/headstart/spring-opacity-scheduling-*.json` artifact. Runtime failures write an artifact with `status: failed` when the application can still report. `--spring --limit-fps` is unsupported and exits with an error before the GUI starts.

The verifier writes a unique `target/headstart/spring-opacity-verification-*.json` report even when a mode fails. It records the binary SHA-256, each result artifact SHA-256, each process exit code, stdout, stderr, validation errors, and an aggregate pass/fail status. The verifier returns a nonzero exit code on failure. Run its focused contract tests with `python3 -m unittest scripts/test_verify_spring_probe.py`.

The first native run produced complete probe artifacts in both modes, but the original verifier rejected their `0.15000000596046448` serialized `f32` target against decimal `0.15` by exact equality. Those artifacts and `target/headstart/milestone60-probe-verification.log` remain as failure evidence. The verifier now accepts finite numeric targets within `1e-7` of the requested sequence and rejects booleans, nonnumeric values, NaN, and infinity.

## What it measures

The probe waits for a stable initial host lowering count, clears diagnostics, and changes the target from `0.15` to `1.0` with one explicit root notification. It passively samples diagnostics for 250 ms, changes the target back to `0.15` with a second explicit notification, and passively samples for another 250 ms. It then waits until at least 2 seconds after retargeting and compares two snapshots 750 ms apart. The sampler only reads snapshots; it does not notify a view or request a frame.

Normal motion passes when root host lowerings grow in both passive motion windows and stop growing in the final window. Reduced motion passes when the two explicit target changes each cause a lowering, while their following passive windows and the final window remain flat. The JSON records actual monotonic event and sample times, requested and effective reduced-motion state, counts, checks, and any error.

The diagnostic counts measure RustUI root host lowerings. They are not an exact GPUI frame count and do not establish presented alpha, spring position, or velocity continuity. A visual retarget check can use time-stamped captures of the existing Playground spring gallery; its visual result should be reported separately from this scheduling probe.
