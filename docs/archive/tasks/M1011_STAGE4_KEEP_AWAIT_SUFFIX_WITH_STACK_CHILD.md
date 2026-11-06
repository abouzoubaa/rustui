# M1011 — Stage 4: keep `.await` with its stack child

The view-builder parser previously ended a child at its constructor call, leaving a following `.await` as an invalid adjacent child. It now keeps the postfix await suffix with the constructor and preserves conditional attributes on the expanded child.

The focused macro regression failed before the parser fix and passed afterward. Current validation: `./scripts/cargo-headstart test -p rustui-macros --lib await_suffix_stays_with_its_constructor_before_the_next_sibling --locked --offline` passed (1/1).

This covers the parser boundary only; broader macro and native Playground acceptance remain open.
