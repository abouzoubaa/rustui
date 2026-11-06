# M774 — Stage 2 / keep conditional siblings' retained identities stable

Implicit retained identities now count occurrences per base identity instead
of using one render-wide slot. A conditional child with a different base can
appear or disappear without renumbering later retained children. Repeated
siblings with the same base still use occurrence order; use explicit IDs when
their order may change.

The focused locked/offline regression passed (1/1), covering conditional
disappearance/reappearance and same-base sibling separation. Scoped formatting
and `git diff --check` passed. Native state-retention and broader lifecycle
acceptance remain open.
