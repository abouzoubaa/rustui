# Breadcrumb stable identity

`BreadcrumbView` must not derive retained identity only from its item count and
first label: sibling paths can share those values while representing different
content. Implicit Breadcrumb IDs use `RenderContext`'s owner-scoped sibling-slot
allocator. Re-rendering the same sibling slot preserves its identity, while
other Breadcrumb siblings receive distinct IDs.

Caller-provided `.id(...)` values are preserved verbatim. Empty and
whitespace-only explicit IDs are rejected. This is source/test evidence; native
focus, activation, VoiceOver, and cross-platform acceptance remain separate.

Focused evidence: `target/headstart/evidence/m515-breadcrumb-sibling-identity-tests-20261021.log`.
