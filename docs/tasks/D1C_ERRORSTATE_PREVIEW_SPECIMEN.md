# D1c ErrorState preview specimen

The canonical compiled-preview gallery exposes an opt-in `error-state-specimen`
with a visible title (1–120 characters) and supporting detail (up to 240
characters). The factory uses the public RustUI `ErrorState` API. Whitespace-only
titles are rejected on import so the alert keeps a nonempty accessible name;
whitespace-only details are omitted from rendering. The descriptor declares no
events under the signal-only D1b preview contract.

The focused regression covers schema bounds and types, defaults, no events,
blank-detail handling, invalid-title rejection without changing the mounted
preview, and descriptor import/export round-trip. The isolated locked/offline
Playground library suite passed 41/41, including this regression. Evidence:
`target/headstart/evidence/roadmap-validation-m251-playground-lib.log`. No GUI or
native accessibility acceptance was run.
