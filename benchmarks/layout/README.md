# layout benchmarks

The GPUI test-window workload measures an owner update alone and an owner update
followed by an additional forced draw, for vertical stacks of 10, 100, and 1,000
text rows:

```sh
./scripts/cargo-headstart test --release --locked --offline -p rustui --lib owner_update_and_draw_workload -- --ignored --nocapture
```

Each size and mode has a warm-up and ten measured batches of ten updates. The runner
reports raw wall time and render calls for each phase and verifies the final
owner value. Record source revision, compiler, profile, machine and full output
for comparisons. This test-window workload includes component lowering and test
draw work; it is not an OS presentation, GPU, scrolling, frame-time, or isolated
layout measurement. Sizing, remeasurement, retained layout and native frame
performance remain open workloads.
