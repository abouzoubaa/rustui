# M1014 — Stage 2: reject overflowing Grid gap spans

Grid now checks that cumulative horizontal gaps across explicit columns and cumulative vertical gaps across occupied rows remain representable as finite `f32` values before lowering the children. This handles finite spacing values whose aggregate span would overflow.

The focused regression verified rejection above the representable span and acceptance at one gap. `./scripts/cargo-headstart test -p rustui --lib rejects_finite_spacing_when_grid_gap_span_overflows` passed (1/1); scoped `git diff --check` passed.

This is a source-level guard; native visual and cross-platform geometry acceptance remain open.
