# M1180 — Stage 2 / reject stack gap count rounding overflow

The shared HStack/VStack gap-span check converted the number of gaps to `f32`
before multiplying. Counts above `2^24` can round down, allowing an aggregate
gap larger than `f32::MAX` to appear finite.

`validate_gap_span` now keeps the exact `usize` count and compares the product
in `f64` against `f32::MAX`. The regression uses a large count and a finite
spacing value whose old rounded count passed while the true count overflowed.
The focused test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui --lib rejects_total_gap_when_f32_rounding_understates_gap_count
```

Animation timing/retargeting, broader lifecycle behavior, and native layout
acceptance remain open.
