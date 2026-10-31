# M967 — Tree item accessibility semantics

Stage 8 / data-component accessibility.

Each rendered Tree row now emits AccessKit `TreeItem` semantics with its
one-based hierarchy level, selected and disabled state, and expanded state
when the row is a folder. Leaf rows omit expanded state.

The focused AccessKit serialization regression passed 1/1 and the scoped
whitespace check passed. Whole-file rustfmt still reports two pre-existing
formatting regions outside this change. Native keyboard and screen-reader
acceptance remain open.
