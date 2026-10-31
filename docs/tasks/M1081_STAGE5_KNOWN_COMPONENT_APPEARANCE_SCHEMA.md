# M1081 — Constrain known component appearance selectors

For each known non-Button component record in the canonical theme schema,
restrict `styles[].when.appearance` to `any`, `light`, or `dark`. This matches
the runtime selector validator while leaving unknown/custom component records
open to their own fields.

The focused Rust schema regression passed 1/1. This is schema/runtime contract
alignment only; native visual and accessibility acceptance remain open.
