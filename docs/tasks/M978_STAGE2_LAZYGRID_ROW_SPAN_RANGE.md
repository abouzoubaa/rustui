# M978 — Stage 2 / reject non-finite LazyGrid aggregate row spans

LazyGrid validates that row height plus row gap remains finite, then ensures
the aggregate span across virtual rows fits in the finite `f32` range.

Focused check: `lazy_grid::tests::total_row_span_must_remain_finite` passed
(1/1).

This source regression does not constitute GUI or platform acceptance.
