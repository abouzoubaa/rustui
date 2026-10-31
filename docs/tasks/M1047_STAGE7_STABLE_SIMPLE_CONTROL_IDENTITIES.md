# M1047 — Stage 7 stable Toggle, Checkbox, and Radio identities

Toggle, Checkbox, and Radio formed their implicit retained identity from the
visible label. When a label changed with the locale, the component's owner slot
received a different GPUI identity, which could discard retained focus or
interaction state across the rerender. Their implicit identities now use the
stable control kind plus the existing owner/sibling slot. Explicit IDs remain
authoritative.

The focused regression verifies that each control keeps its identity across a
same-slot label localization change and that same-kind siblings remain distinct:

```sh
./scripts/cargo-headstart test -p rustui --lib identity_survives_label_localization_and_distinguishes_siblings
```

Result: 3 passed. SearchField delegates to TextField, whose explicit
placeholder-localization and sibling-identity regression also passed 1/1.
The pre-existing shared TextField/TextEditor identity test passed 2/2 as an
additional check. Scoped rustfmt and whitespace checks passed.

This verifies identity derivation at source level. Native locale switching,
focus continuity, interaction, screen-reader output, RTL layout, and
cross-platform acceptance remain open.
