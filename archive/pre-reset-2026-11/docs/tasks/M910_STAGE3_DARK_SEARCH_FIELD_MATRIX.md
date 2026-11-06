# M910 — Stage 3 / Dark Search Field state matrix

## Change

The canonical Input gallery now presents all six combinations specified by
`apps/rustui-playground/references/Search Field.svg`: Disabled, Focused, and
Idle, each with text absent or present. The M905 populated-disabled specimen
keeps its existing ID and controlled initial value and is included in this
matrix. The four enabled cases have independent editable query state. Focused
examples are ordinary Search Fields; focus them manually to observe the native
focused appearance. The gallery labels the matrix and asks the operator to
switch the Playground appearance to Dark.

The catalog now describes the six represented combinations, and the obsolete
six-entry `MISSING_REFERENCES` list is empty. Search Field coverage remains
Partial pending native visual and accessibility acceptance.

## Validation

Focused gallery tests passed 2/2:

```sh
./scripts/cargo-headstart test -p rustui-playground --lib stories::canvas::search_field_reference_tests --locked --offline
```

The tests check the complete 3×2 state matrix, preserve the M905
populated-disabled specimen identity, and verify the independent query
defaults. Native dark appearance, focus visuals, interaction, and accessibility
acceptance remain unverified.
