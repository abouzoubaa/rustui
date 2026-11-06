# Stage 9 / D1c ZStack preview specimen

## Scope

The compiled preview gallery registers an opt-in `zstack-specimen` for the
public RustUI `ZStack` layout. Its descriptor exposes one schema-constrained
alignment property (`center`, `top-leading`, or `bottom-trailing`). A
compiled-local factory builds a fixed, two-layer composition inside a bounded
panel; the descriptor does not serialize view trees, callbacks, closures, or
retained layout state. The specimen emits no events or actions.

## Validation

- Focused locked/offline codec and gallery regression passed: `1 passed; 0
  failed`. It verifies the default, the allowed alignment set, a descriptor
  round trip, and rejection of an unsupported alignment. Log:
  `target/headstart/evidence/d1c-zstack-specimen-test-20261021.log`.
- Locked/offline Playground all-targets check exited 0. Log:
  `target/headstart/evidence/d1c-zstack-specimen-check-20261021.log`.
- Edition 2024 rustfmt and scoped `git diff --check` passed.

Native rendering, alignment behavior, keyboard and screen-reader behavior,
locale/RTL, and cross-platform acceptance remain open.

## Final mount-path check (2026-10-21)

The specimen regression now calls the gallery's actual `mount` path directly;
the redundant test-only mount helper was removed. The final focused test passed
1/1 and the locked/offline Playground all-targets check exited 0 without the
unused-helper warning. Logs: `target/headstart/evidence/d1c-zstack-specimen-test-final-20261021.log` and
`target/headstart/evidence/d1c-zstack-specimen-check-final-20261021.log`.
