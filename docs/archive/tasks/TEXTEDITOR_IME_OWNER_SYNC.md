# TextEditor owner synchronization during local editing

When a retained TextEditor rerenders while its owner binding is unchanged, the
retained editor value remains authoritative. This preserves local editing state
such as an in-progress IME composition. If the owner value changes, the new
owner value is applied unless the retained editor already contains it.

`TextareaState` exposes the active marked-text range through GPUI's
`EntityInputHandler` contract. Reconciliation preserves a preedit while the
owner is unchanged, applies external owner changes during composition, and
restores the unchanged owner after a canceled composition clears a previously
observed marked range. It tracks that transition so ordinary local edits with
no marked text remain authoritative. Native IME composition and cross-platform
behavior remain unverified.

## Earlier validation

- `./scripts/cargo-headstart test --quiet -p rustui --lib text_editor --locked --offline` — passed, 11 tests, 0 failed.
- `rustfmt --edition 2024 crates/rustui/src/components/input/text_editor.rs` — passed.
- `git diff --check -- crates/rustui/src/components/input/text_editor.rs` — passed.

Evidence: `target/headstart/evidence/m587-text-editor-ime-owner-sync-test-20261021.log`.

## M702 — make owner-sync transitions explicit

The existing renderer already advanced its remembered owner value whenever the
binding changed, including when the retained editor value already matched the
new owner value. The focused regression did not cover that branch. Owner
synchronization is represented by an explicit plan so the two independent
actions remain visible: update the editor only when needed, and advance the
owner snapshot on every owner change. The regression covers unchanged-owner
IME preedit, external updates, an owner value already present in retained state,
and a later local edit after that snapshot advances.

The focused locked/offline RustUI test passed 1/1 after formatting. This is
source-level contract evidence; native IME and cross-platform acceptance remain
unverified. Evidence: `target/headstart/evidence/m702-text-editor-owner-sync-plan.log`.

## Follow-up — restore canceled IME preedit

The retained editor previously preserved any divergent value whenever the owner
was unchanged, even after GPUI cleared the marked range for a canceled
composition. It now tracks whether a marked range was active and restores the
unchanged owner only on the transition from marked text to no marked text.
External owner updates still replace preedit text, and ordinary local edits
without a marked range remain authoritative. The focused locked/offline test
passed 12/12; rustfmt passed. Native IME behavior remains unverified.
