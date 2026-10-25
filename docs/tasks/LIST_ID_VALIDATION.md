# List retained identity validation

`ListView::id` supplies the retained `ListState` identity. Reject empty and
whitespace-only values so an invalid explicit identity cannot alias another
retained list. This follows the corresponding Table, Tree, KeyedList, and
DataGrid contracts.

The focused locked/offline List module suite passed 11/11 tests; Rust 1.97.1
rustfmt passed. Native interaction, keyboard, accessibility, RTL, cross-platform,
and performance acceptance remain separate. Evidence:
`target/headstart/evidence/list-id-validation-focused-20261021.log`.
