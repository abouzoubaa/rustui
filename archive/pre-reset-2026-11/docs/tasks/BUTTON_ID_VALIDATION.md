# Button element identity validation

`ButtonView::id` rejects empty and whitespace-only explicit IDs before assigning
the GPUI element identity. Valid IDs are retained verbatim, including
surrounding whitespace, so validation does not silently rewrite caller input.

The focused locked/offline Button suite passed 5/5 and the RustUI library check
passed. Rust 1.97.1 rustfmt passed. Native rendering and accessibility acceptance
remain separate. Evidence:
`target/headstart/evidence/button-id-validation-focused-20261021.log` and
`target/headstart/evidence/button-id-validation-check-20261021.log`.

## M527 — implicit sibling identity

When `.id(...)` is absent, Button now derives its GPUI ID through the render
owner's implicit sibling-slot allocator, so repeated labels receive distinct
IDs and keep their slots across rerenders. Explicit IDs still bypass fallback
allocation and remain verbatim. The focused Button suite passed 6/6:
`target/headstart/evidence/m527-button-sibling-identity-tests-20261021.log`.
Native focus targeting and accessibility acceptance remain open.
