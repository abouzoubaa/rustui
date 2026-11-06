# M1213 — Stage 0 correct the SplitView registry status

`docs/FILE_REGISTRY.md` incorrectly labeled `crates/rustui/src/layout/split_view.rs`
“Documentation only.” The row now records the public implementation and M586's
bounded native resize/lifecycle acceptance, while retaining the open focus,
accessibility, locale, and broader platform limits.

Verification: current registry inventory paths resolved, and
`git diff --check -- docs/FILE_REGISTRY.md` passed. No tests were run for this
documentation correction.
