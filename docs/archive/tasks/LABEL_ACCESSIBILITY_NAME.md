# Label accessible text

`LabelView::new` now rejects an empty or whitespace-only primary label. The
RustUI wrapper passes its primary text to pinned GPUI Components `Label`, which
builds its rendered `StyledText` from the primary and optional secondary text;
the wrapper provides no separate accessible-name fallback. With no meaningful
primary or secondary text, a semantic Label can render without a useful text
name. The constructor guard prevents that invalid state before rendering.

The regression test covers a whitespace-only label. Validation passed:

- `./scripts/cargo-headstart test -p rustui --lib components::basic::label::tests::rejects_blank_visible_labels --locked --offline` (1 passed).
- `CARGO_TARGET_DIR="$PWD/target/headstart/label-accessible-name-check-20261020" ./scripts/cargo-headstart check -p rustui --all-targets --locked --offline` (exit 0).

Raw command and output are recorded in
`target/headstart/evidence/label-accessible-name-20261020.log`. This is
source-level validation; no GUI or native accessibility acceptance was run.
