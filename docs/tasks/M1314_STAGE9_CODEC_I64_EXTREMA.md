# M1314 — Stage 9: preserve i64 extrema in the preview codec

## Contract

Preview integer properties use the full `i64` domain. JSON decoding and
re-encoding must preserve `i64::MIN` and `i64::MAX` as exact integers, and
tokens immediately outside that range must be rejected as invalid integer
properties.

## Change

Added a codec regression that decodes and re-encodes both extrema, checks the
integer values and emitted numeric tokens, then rejects positive and negative
one-past-range tokens. No production implementation change was needed.

## Verification

The focused locked/offline test passed 1/1, and the complete
`preview::codec::tests` module passed 8/8. Full command output is recorded in
`target/headstart/evidence/m1314-preview-codec-i64-extrema-20261024.log`.

```sh
./scripts/cargo-headstart test -p rustui --lib preview::codec::tests::i64_extrema_round_trip_and_out_of_range_json_integers_are_rejected --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --exact --quiet
./scripts/cargo-headstart test -p rustui --lib preview::codec::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1304"' -- --quiet
```

The temporary vendor overlay used a replacement `cc-1.8.0` package from
worktree 6390 after all 36 manifest-listed files matched their checksums. The
overlay symlinks were removed afterward; no vendor files were changed and
`target/headstart` was preserved.

This is local descriptor codec evidence; native and broader preview
acceptance remain separate. D1d composition and D1b IME ownership decisions
remain gated.
