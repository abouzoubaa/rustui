# State-driven SpringOffset

`SpringOffset::new(id, child, x, y)` applies finite signed logical-pixel target
coordinates using two GPUI `SpringAnimation<Pixels>` values and GPUI's retained
`with_spring` scheduling. A stable ID retargets native position/velocity;
replacing the ID creates a fresh spring at its target. GPUI owns timing,
settling and app-level reduced-motion behavior. No RustUI animation clock is
introduced.

The nested native elements preserve the child's layout slot. Their relative
position changes the child bounds GPUI uses for hit testing and accessibility
geometry. Clipping, overlaps, stacking, and accessibility-tree behavior remain
controlled by GPUI and ancestors. This is translation only; scale/rotation and
exact frame, pixel, interruption-velocity, screen-reader, and performance
acceptance are not provided by source tests.

Focused coordinate validation passed 1/1 with
`./scripts/cargo-headstart test -p rustui animation::offset::tests::validates_each_axis_and_updates_targets --locked --offline`.
The locked/offline Playground all-targets check exited 0. Logs are retained at
`target/headstart/evidence/spring-offset-focused-test.log` and
`target/headstart/evidence/spring-offset-playground-check.log`; rustfmt and
`git diff --check` passed. No native GUI run was performed.

The Playground Offset gallery reuses its existing state controls and moved
interactive subtree to demonstrate target retargeting. See
`apps/rustui-playground/src/stories/offset_gallery.rs` and the unit test in
`crates/rustui/src/animation/offset.rs`.


## M472 — Reject blank retained spring identities

`SpringOffset::new` now returns `SpringOffsetError::EmptyIdentity` for empty or
whitespace-only IDs while preserving nonblank IDs verbatim. The same contract is
applied to `SpringOpacity::new` through `SpringOpacityError::EmptyIdentity`;
these IDs directly name GPUI retained springs and child scopes. Finite
coordinate/opacity and physics validation remain unchanged.

Focused locked/offline identity tests cover both wrappers, empty and
whitespace-only inputs, and spelling-preserving valid IDs. Native motion,
visual, focus, accessibility, and cross-platform acceptance remain open.

Focused M472 verification passed: `animation::offset::tests` 2/2 and
`animation::spring::tests` 4/4, both with locked/offline Headstart Cargo. Evidence
logs: `target/headstart/evidence/m472-spring-offset-identity-tests-20261021.log`
and `target/headstart/evidence/m472-spring-opacity-identity-tests-20261021.log`.
