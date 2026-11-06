# state benchmarks

Run the owner-binding microbenchmark from the repository root:

```sh
./scripts/cargo-headstart bench --locked --offline -p rustui-core --bench binding_update
```

The runner warms both cases, then emits ten raw CSV samples for changed and
unchanged `Binding::update_on` calls, alternating their order. Each sample checks
the final owner value and write count outside its timed interval. The default is
one million calls per sample; `RUSTUI_BENCH_ITERATIONS` accepts 1–10,000,000.
Record source revision, compiler, profile, features, machine and raw output when
comparing runs. The operation is an in-process binding dispatch microbenchmark:
it does not measure GPUI notifications, derived values, rendering or frame time.
Those workloads and release performance budgets remain open.
