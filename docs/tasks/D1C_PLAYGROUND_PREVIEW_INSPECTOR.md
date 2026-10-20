# D1c canonical Playground preview inspector

## Contract

The Layout story's compiled preview gallery is the canonical local inspector for
the public `rustui::preview` API. It discovers editable properties from the
mounted component schema and creates stable `PropertyInspector` rows for the
string, Boolean, and Number properties. String values use text fields,
Boolean values use toggles, and Number values use a decimal text draft parsed
and validated as `f64` by the preview host. Incomplete or out-of-range numeric
drafts remain visible with feedback while the last accepted numeric value stays
in the host. Writes use the preview host's
stamp-checked binding feedback path. The displayed descriptor can be imported
through D1a validation and exported as normalized JSON; import preserves the
descriptor's preview ID and remounts it as a new generation. The Button specimen
continues to resolve its signal-only `activate` event through its allowlisted
owner action.

Import and Export controls precede the multiline JSON editor so keyboard users can reach them first. The editor also binds `secondary-shift-enter` to the owner-scoped Import action, so edited JSON can be imported without leaving the editor. The pinned Textarea keeps Tab and Shift-Tab for indentation.

Export copies the preview document and overlays the live host's resolved
property map before encoding. This ensures accepted binding edits are represented
even when the compiled view was created from an earlier immutable render
snapshot; schema defaults are included in the normalized exported descriptor.

Factories and owner state remain compiled/local. The panel does not serialize
Rust views, callbacks, bindings, or GPUI entities. Theme documents remain a
separate format. The three local specimens share editable `text`, `enabled`, and fractional
`amount` properties so Boolean and numeric edits are visible in rendered output
and owner state. This does not establish broad component-catalog coverage or
arbitrary type-specific editors.

## Implementation and validation

Implementation: `apps/rustui-playground/src/stories/compiled_preview_gallery.rs`.
The Playground all-targets locked/offline check passed through
`./scripts/cargo-headstart check -p rustui-playground --all-targets --locked --offline`.
The locked/offline Playground build also passed through
`./scripts/cargo-headstart build -p rustui-playground --locked --offline`.
Compiler output included pre-existing unused-import/dead-code warnings in
`rustui`, `canvas.rs`, and `catalog.rs`; the D1c gallery's obsolete hard-coded
edit action was removed after schema-driven editing replaced it.

An interactive Playground check exposed a stale-export case after editing the
schema inspector's Text field. Export was corrected to use the live resolved
property map; rerun the interactive check after rebuilding to confirm the
exported `text` value matches the accepted edit. No new GUI run or application
packaging was performed for this source slice. Native
theme, locale, import failure recovery, lifecycle, keyboard, accessibility and
specimen-retention acceptance remain open. A build/check result does not establish
native Playground acceptance.


Interactive Playground verification (2026-10-19): the schema inspector and compiled TextField both displayed RoundTrip. Export produced normalized JSON with properties.text = RoundTrip. Import preserved preview ID compiled-gallery and remounted at generation 2, revision 0. Importing malformed JSON was rejected; the live RoundTrip value and generation 2, revision 0 mount remained intact. The valid descriptor was re-exported afterward. Playground was quit, its process was confirmed absent, and the disposable bundle and executable were removed. Broader lifecycle, keyboard, accessibility, theme, locale, and cross-platform acceptance remain open.


Keyboard traversal follow-up (2026-10-19, M138): the canonical macOS 27
Playground allowed Tab traversal from the inspector field through Rerender,
Teardown, Import, and Export, then into the JSON editor. Return activated Export
from keyboard focus and the normalized JSON contained `properties.text` equal to
`Keyboard value`. The Import button received keyboard focus in the same order,
but keyboard activation of Import was not confirmed in this run. Import after
editing JSON remains unreachable by Tab while the multiline editor owns focus.
See `target/headstart/evidence/d1c-keyboard-m138-20261019.json`.


Keyboard Import follow-up (2026-10-19, M140): with the JSON editor focused, Command-Shift-Return rejected malformed JSON while preserving generation 1 and the mounted value. A valid descriptor then imported through the same shortcut, preserved preview ID `compiled-gallery`, advanced to generation 2, and updated both the owner value and compiled TextField to `Keyboard Import`. Evidence: `target/headstart/evidence/d1c-keyboard-import-m140-20261019.json`. Text selection, IME, accessibility, locale, and cross-platform acceptance remain open.


Typed property follow-up (2026-10-20, M144): the inspector renders schema-derived Boolean toggles and decimal `f64` rows. On macOS 27, `enabled` toggled false in owner and preview output; `amount` accepted `0.12345678901234566` without integer or `f32` conversion. Normalized export and import preserved both values and advanced the generation. An out-of-range `1.5` draft was rejected while the accepted host value remained unchanged. After remount, a new-generation edit to `0.246813579` succeeded at revision 1; generation-scoped row identities avoid reusing a prior mount's captured stamp lease. Locked/offline all-targets check and Playground build passed. Evidence: `target/headstart/evidence/d1c-typed-inspector-m144-20261020.json`. VoiceOver, IME, selection, theme/locale, RTL, cross-platform, and broader catalog acceptance remain open.


Stale-callback correction (2026-10-20, M145): `write_amount_draft` checks the captured stamp against the mounted preview before mutating the draft. Stale generation or revision callbacks return without changing draft state. The locked/offline all-targets check, Playground build, and `git diff --check` passed. M149 replayed a retained generation-1 `Binding<String>` after import remounted generation 2. The old callback returned false and left the accepted draft and host amount unchanged; a current-generation edit then succeeded. This deterministic replay does not claim a naturally queued GPUI event or race. Evidence: `target/headstart/evidence/d1c-stale-draft-guard-m145-20261020.json` and `target/headstart/evidence/d1c-stale-draft-replay-m149-20261020.json`.


M149 native replay (2026-10-20): the canonical macOS 27 Playground captured the amount `Binding<String>` at generation 1/revision 0, imported the descriptor to remount at generation 2/revision 0, then accepted `0.246813579` at revision 1. Replaying the retained old binding with `0.987` returned false; the visible draft, owner amount, and compiled preview remained `0.246813579` at revision 1. A current-generation edit to `0.13579` succeeded at revision 2. This deterministic `Binding::write_to` replay verifies the D1c captured writer and guard, not naturally delayed GPUI event dispatch. Locked/offline all-targets check, Playground build, rustfmt check, and `git diff --check` passed; tests were not run. The Playground process was stopped, the disposable bundle moved to Trash, and shared Cargo artifacts were preserved. Evidence: `target/headstart/evidence/d1c-stale-draft-replay-m149-20261020.json`. Native IME, VoiceOver, theme/locale, RTL, cross-platform and broader catalog acceptance remain open.

## M149 ordered replay follow-up

The canonical macOS 27 sequence was repeated with the stale callback replayed immediately after descriptor import, before any generation-2 edit. The generation-1 callback returned false and left the draft, owner value, and preview at `0.246813579`, generation 2/revision 0. A current-generation edit to `0.13579` then succeeded at revision 1. The first run, which edited the current generation before replay, is retained alongside the repeat in the evidence JSON. This deterministic callback replay still does not demonstrate naturally queued GPUI event dispatch. Evidence: `target/headstart/evidence/d1c-stale-draft-replay-m149-20261020.json`.
