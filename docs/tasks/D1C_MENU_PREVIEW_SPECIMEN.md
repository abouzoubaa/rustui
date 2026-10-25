# D1c Menu preview specimen

The canonical compiled-preview gallery exposes an opt-in public RustUI `Menu`
specimen. Its only descriptor property is a bounded `trigger_label`; the
signal-only `select` event has no payload and is explicitly mapped to the
registered `increment` action. The compiled menu action captures the mounted
`PreviewSignal` through `OwnerAction::action_with`, then routes it through
`PreviewHost::dispatch`, which enforces the current stamp and catalog allowlist.
The trigger ID includes the preview generation.

The focused regression checks the schema bounds, empty event payload, allowlist
mapping, current-signal dispatch, stale-signal rejection after a property edit,
and descriptor round-trip. The focused test and isolated all-targets check
passed; evidence is recorded at
`target/headstart/evidence/d1c-menu-preview-tests-20261020.log` and
`target/headstart/evidence/d1c-menu-preview-check-20261020.log`.

This source-level check does not establish native menu selection, keyboard
behavior, accessibility, localization, or cross-platform acceptance. Event
payloads and arbitrary callbacks remain outside the signal-only preview
contract.
