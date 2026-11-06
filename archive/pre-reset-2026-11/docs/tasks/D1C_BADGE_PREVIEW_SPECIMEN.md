# D1c Badge preview specimen

The canonical compiled-preview gallery includes an opt-in `badge-specimen`. Its
serializable properties are a visible label (1–80 characters), an integer count
(0–999), and a dot-mode Boolean. The factory uses the public RustUI `Badge` and
`Text` APIs; dot mode selects the native dot variant, while count mode uses the
bounded count. The specimen declares no events under the signal-only preview
contract.

The focused regression covers schema bounds and types, defaults, both render
modes, and descriptor export/import round-trip. It passed 1/1. An isolated
locked/offline Playground all-targets check also passed. Rustfmt and scoped diff
checks passed. Existing dead-code and future-incompatibility warnings remain.

Evidence: `target/headstart/evidence/d1c-badge-specimen-test-20261020.log` and
`target/headstart/evidence/d1c-badge-specimen-check-20261020.log`. No GUI or native
accessibility acceptance was run.


## Gallery reachability correction — 2026-10-20

Review found the specimen schema and mount method were present, but the gallery had no visible control wired to mount it. A `Preview Badge specimen` control is now present beside Avatar and ColorPicker. The isolated locked/offline workspace all-targets check and Playground library suite pass; the suite reports 41/41 tests. Evidence: `target/headstart/evidence/roadmap-m254-workspace-check-20261020.log` and `target/headstart/evidence/roadmap-m254-playground-lib-20261020.log`. No GUI or native accessibility acceptance was run.
