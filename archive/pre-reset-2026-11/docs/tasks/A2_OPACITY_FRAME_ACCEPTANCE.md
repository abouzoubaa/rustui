# M35 runtime frame acceptance

`crates/rustui/examples/opacity_benchmark.rs` measures the `OpacityProbe`
`ViewHost` through the existing `render_diagnostics` API. Run the binary twice
in separate fresh processes:

```sh
./scripts/cargo-headstart build -p rustui --example opacity_benchmark
python3 scripts/verify_opacity_probe.py
```

The runner starts the built binary twice, once per mode, with an independent
10-second hard timeout for each process. It validates each result's mode,
settings, sample timing, measurement scope, and required count progression. A
unique `opacity-probe-verification-*.json` record under `target/headstart/`
retains both probe JSON results, output streams, exit codes, and validation
errors, including timeout or startup failures. Pass `--binary PATH` to use a
probe executable outside the default Headstart target path. The record also
stores SHA-256 hashes for the executed binary, probe source, and verifier source,
along with UTC run timestamps and the exact mode arguments. Existing evidence
records are retained under unique filenames.

Each process first shows static content. A read-only timer poll waits up to 3
seconds for the root host's lowering count to remain stable for 750 ms. The
probe then clears diagnostics and sends one controlled root update to mount the
1-second one-shot fade. Its first nonzero lowering is the animation baseline;
the measured timer polling interval must be at most 250 ms. Midpoint, post-
duration, and quiescence samples follow 500 ms, 1.2 seconds, and 1.5 seconds
after that baseline. Sample and mount timestamps use a monotonic clock from
window setup. The app records requested and effective reduce-motion values;
`reduce_motion_confirmed` means they match and is expected to be true in both
modes.

The timer sampler only reads snapshots. It never notifies a view, refreshes the
window, or requests frames. The example has a 10-second in-app timeout that
writes a failure result, and the Python runner applies its own 10-second process
timeout.

Accept normal motion when root lowering counts rise from baseline to midpoint,
rise again by the post-duration sample, and remain unchanged at quiescence.
Accept reduced motion when the static startup phase stabilizes, the requested
and effective flags match, and all post-mount counts remain equal to baseline.
GPUI's one-shot reduced-motion contract resolves the target to `1.0`; this
harness does not inspect the computed opacity or captured pixels.

The counters measure RustUI host lowerings. They show repeated host lowering
during normal animation and stable counts after its duration. They are not an
exact count of GPUI frame callbacks, GPU frames, presented frames, or presentation
timing. Existing endpoint screenshots remain the visual evidence. Result JSON
files are written to `target/headstart/`.

The initial verifier record,
`target/headstart/opacity-probe-verification-19330-1792317490354016000.json`,
failed both modes because baseline timing was invalid; the reduced-motion run
also showed a further host lowering. That result remains retained as a failed
execution, not acceptance evidence.

The corrected probe and verifier completed in separate normal and reduced-motion
processes. The final record is
`target/headstart/opacity-probe-verification-31323-1792319071087036000.json`.
Normal root host lowerings were baseline 3, midpoint 23, post-duration 43, and
quiescence 43. Reduced-motion lowerings remained 1 at all four samples. Both
processes exited 0, had no verifier errors, and confirmed their requested motion
mode. This accepts the stated RustUI host-lowering scheduling checks. It does
not count GPUI callbacks, GPU or presented frames, measure presentation timing,
inspect intermediate pixel alpha, or establish OS reduced-motion propagation.
