# Stage 9 / D1c VStack preview specimen

The canonical compiled-preview gallery registers an opt-in specimen for the
public RustUI `VStack` layout. Its schema exposes editable numeric `spacing`
bounded to 0–32 logical pixels and defaults to 12. The compiled-local factory
keeps three centered Text children fixed and maps the descriptor value to
`VStack::spacing`; child structure and callbacks are not serialized.

The focused regression checks the bounded schema and empty event/action maps,
then imports a descriptor with spacing 24.5 and verifies the mounted value.
Locked/offline focused test passed 1/1. The Playground all-targets check and
edition-2024 rustfmt check passed. Evidence:

- `target/headstart/evidence/d1c-vstack-specimen-test-20261021.log`
- `target/headstart/evidence/d1c-vstack-specimen-check-20261021.log`
- `target/headstart/evidence/d1c-vstack-specimen-rustfmt-20261021.log`

No GUI acceptance was run. Native layout appearance, keyboard and
screen-reader behavior, localization/RTL, and cross-platform acceptance remain
open.
