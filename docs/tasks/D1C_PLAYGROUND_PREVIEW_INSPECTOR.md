# D1c canonical Playground preview inspector

## M637 — Reject stale text edits before property validation

`write_text` now compares the captured binding stamp with the current preview snapshot before date/nonblank validation can mutate the inspector status. A stale blank-label callback leaves status and mounted value unchanged. Focused test `stale_text_binding_does_not_change_preview_status_or_value` passed 1/1.

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

## Toggle catalog extension (2026-10-20)

The canonical compiled-preview gallery now registers the public RustUI Toggle as
an opt-in specimen. Its descriptor has only the actual editable inputs: string
`label` and Boolean `checked`. A compiled factory renders `Toggle` with a
mount-scoped owner binding; accepted changes feed the `checked` property through
the existing stamp-validated preview-host binding path. Descriptor import maps
those fields back to the gallery owner, and the Playground exposes a
`Preview Toggle` control. Signal-only event behavior and the descriptor format
are unchanged.

A focused Playground unit test verifies the schema fields and checked-value
feedback through `PreviewHost`; it passed. The fresh isolated locked/offline
Playground all-targets check also passed using
`target/headstart/localization-preview-check`. Its log is
`target/headstart/evidence/playground-interpolation-check-20261020.log`; the
matching `rustui-i18n` test run passed 11 unit and 1 integration test, recorded
in `target/headstart/evidence/i18n-interpolation-tests-20261020.log`. A later
check against the shared `target/headstart` still returned E0432 for the
localization exports even though current sources export them. Treat that
shared-target result as an inconsistent artifact/cache observation; the clean
isolated checks are the current build evidence. With the same isolated target,
`check --workspace --all-targets --locked --offline` and
`test --workspace --all-targets --locked --offline` both passed. The workspace
test run passed 361 tests across nine nonempty result groups. Logs:
`target/headstart/evidence/roadmap-workspace-all-targets-toggle-check-20261020.log`
and `target/headstart/evidence/roadmap-workspace-all-targets-toggle-tests-20261020.log`.
No native interaction acceptance was run for Toggle, so rendered appearance,
pointer/keyboard operation, accessibility, locale, RTL, and cross-platform
behavior remain unverified for this new specimen.

## Checkbox catalog extension (2026-10-20)

The canonical compiled-preview gallery now registers the public RustUI Checkbox
as an opt-in specimen. Its descriptor exposes only the actual component inputs:
string `label` and Boolean `checked`. The compiled factory renders `Checkbox`
with the existing mount-scoped owner binding, and accepted changes pass through
the preview host's stamp-validated binding feedback. Descriptor import restores
the same label/checked fields, and the Playground exposes a `Preview Checkbox`
control. No event payload or descriptor-format changes were made.

A focused Playground unit test verifies the schema and checked-value feedback.
Locked/offline validation uses the isolated
`target/headstart/localization-preview-check` cache. This source-level check
does not establish native Checkbox interaction, accessibility, locale, RTL, or
cross-platform acceptance.

## Constrained Picker catalog extension (2026-10-20)

Editable string schema properties with non-empty `allowed_values` now use a
choice row backed by RustUI's public `Picker`; unconstrained strings remain text
fields. The compiled gallery adds an opt-in `Picker` specimen with the
`selection` values `compact`, `comfortable`, and `spacious`. Both the specimen
and inspector choices feed the existing preview binding feedback path, so the
host still validates the mounted stamp and rejects values outside the catalog.
The descriptor keeps its existing format: selected values round-trip through
the preview codec, while the registered catalog remains the source of choice
constraints. Both focused tests passed on 2026-10-20: the inspector choice-row
test verifies option retention and writes to its owner binding; the Playground
Picker specimen test verifies allowed-value rejection, stamp-checked feedback,
and descriptor export/import round-trip. A fresh-target locked/offline
Playground check also passed; its log is
`target/headstart/evidence/picker-playground-check-20261020.log`.

The focused tests used
`CARGO_TARGET_DIR=/Users/Abium/Developer/rustui/target/headstart/diagnostics/localization-20261020`:

- `./scripts/cargo-headstart test -p rustui --lib components::advanced::property_inspector::tests::choice_rows_keep_options_and_forward_selection_to_the_owner --locked --offline` — 1 passed, 0 failed.
- `./scripts/cargo-headstart test -p rustui-playground --lib stories::compiled_preview_gallery::tests::picker_specimen_uses_constrained_selection_and_round_trips --locked --offline` — 1 passed, 0 failed.

Native Picker interaction and accessibility acceptance for the new specimen
remain pending.

## Radio catalog extension (2026-10-20, M195)

The canonical compiled-preview gallery registers one opt-in RustUI Radio
specimen with editable `label` and Boolean `selected` properties. It uses the
existing stamp-checked binding feedback and descriptor import/export path. The
schema's event map remains empty under the signal-only D1b contract. Its
focused round-trip test passed 1/1. This exercises one Radio control; it does
not define or verify a Radio group. Native interaction, accessibility, locale,
RTL, and cross-platform acceptance remain open. See
`apps/rustui-playground/src/stories/compiled_preview_gallery.rs` and
`target/headstart/evidence/radio-preview-test-m195-20261020.log`.

## Slider catalog extension (2026-10-20, M199)

The gallery now registers an opt-in RustUI Slider specimen with one editable
numeric `value` property constrained to `[0, 1]`, default `0.5`, and no event
payloads under the signal-only preview contract. It uses the existing
generation-stamped owner binding, routes accepted feedback through the preview
host, exposes the accessible name “Preview amount,” and imports/exports through
the existing descriptor codec.

The Slider control accepts `f32`, while the descriptor property is `f64`. The
control binding adapts the owner value to `f32` for rendering and leaves the
imported `f64` untouched in the host and descriptor until interaction. On user
feedback, the selected `f32` value becomes its exact `f64` representation. Two
focused tests cover stamp-checked feedback, stale callback rejection,
round-trip, and importing/exporting `0.1` before interaction. They passed; the
isolated locked/offline Playground all-targets check, rustfmt, and diff checks
also passed. No GUI check ran for this specimen. Native interaction and
screen-reader acceptance remain open.

Evidence: `target/headstart/evidence/d1c-slider-preview-tests-20261020.log` and
`target/headstart/evidence/d1c-slider-inexact-import-check-20261020.log`.

## Stepper catalog extension (2026-10-20, M200)

The preview descriptor now supports an `Integer` property with an `i64` value;
codec round-trip preserves the exact integer, including values that cannot be
represented exactly as `f64`. The retained `PropertyInspector` has an integer
value editor, and the canonical gallery registers an opt-in signal-only Stepper
specimen with bounded integer state, generation/revision-stamped feedback, and
descriptor import/export. The specimen test exercises accepted feedback and
round-trip while keeping the D1b event map empty. Fifteen preview-core tests,
five PropertyInspector integer tests, and the Stepper specimen test passed. The
isolated locked/offline Playground all-targets check passed with existing
warnings. The initial PropertyInspector test run failed compilation on a
missing `AppContext` import and a borrow conflict in its test; the recorded
rerun passed all five tests. A shared-target all-targets attempt failed with
missing localization exports; the isolated check succeeded. No GUI acceptance
was run. Native interaction, accessibility, and broader catalog coverage
remain open.

Evidence: `target/headstart/evidence/integer-preview-core-tests.log`,
`/tmp/rustui-property-inspector-integer-test.log`,
`target/headstart/evidence/d1c-stepper-preview-tests-m200-20261020.log`, and
`target/headstart/evidence/d1c-stepper-playground-check-isolated-m200-20261020.log`.

## Progress catalog extension (2026-10-20)

The canonical compiled-preview gallery registers an opt-in determinate
Progress specimen with editable `label` and numeric `value` properties,
constrained to `[0, 100]`. It uses the public RustUI `Progress` API, keeps the
signal-only event map empty, routes inspector edits through generation-stamped
host feedback, and imports/exports both properties through the existing
descriptor codec. Progress accepts `f32` while the descriptor stores `f64`; the
factory converts the snapshot value for rendering without rewriting the
descriptor. The regression test imports an inexact `f64`, checks that export
preserves it before interaction, verifies accepted value/label feedback and
range rejection, then checks the edited descriptor survives another import.

The focused regression test passed 1/1. The shared-target Playground
all-targets check failed with unresolved `rustui_i18n` localization exports;
the isolated locked/offline all-targets check passed with two existing unused
item warnings and the `block 0.1.6` future-incompatibility notice. Rustfmt and
`git diff --check` passed. No GUI acceptance was run. Native Progress
interaction, accessibility, indeterminate-mode editing, and cross-platform
acceptance remain open.

Evidence: `target/headstart/evidence/d1c-progress-preview-tests-20261020.log`
and `target/headstart/evidence/d1c-progress-preview-check-20261020.log`.

## M212 — TextEditor specimen follow-up (2026-10-20)

The gallery now registers an opt-in signal-only TextEditor specimen with
serializable placeholder and multiline value. Its schema-derived inspector uses
stamp-checked feedback and descriptor export/import. Focused test 1/1 and
isolated locked/offline Playground all-targets check passed; rustfmt and scoped
diff checks passed. GUI acceptance remains unverified. See
`docs/tasks/D1C_TEXTEDITOR_PREVIEW_SPECIMEN.md`.

## M214 — ComboBox specimen follow-up (2026-10-20)

The canonical gallery now contains an opt-in ComboBox schema with a fixed
accessible label and `selection` constrained by `allowed_values`. The signal-only
specimen uses generation-stamped feedback and normalized descriptor
import/export. Focused test 1/1 and isolated locked/offline Playground
all-targets check passed. No GUI acceptance was run. See
`docs/tasks/D1C_COMBOBOX_PREVIEW_SPECIMEN.md`.

## M227 — Dialog specimen follow-up (2026-10-20)

The canonical gallery now includes an opt-in public Dialog specimen. Its
serializable trigger label is editable; title and body remain compiled
content. Confirm and Cancel emit empty-payload signals routed through the
allowlisted owner action, and descriptor import/export remounts the preview
through the existing stamped feedback path. The focused test passed 1/1; the
isolated locked/offline Playground all-targets check, rustfmt, and scoped diff
check passed. No GUI/native acceptance ran. See
`docs/tasks/D1C_DIALOG_PREVIEW_SPECIMEN.md`.


M262 adds a public `Label` specimen with editable primary and secondary text
and a vibrant-style switch. It uses no icon asset, adds no event, and reuses
the existing descriptor inspector. Native rendering and accessibility checks
remain open; see `docs/tasks/D1C_LABEL_PREVIEW_SPECIMEN.md`.


M263 adds a public `Tooltip` specimen with bounded descriptive text and a
compiled `Text` child. It receives a generation-specific identity; its
description does not replace the child's accessible name. Native interaction
and accessibility checks remain open; see
`docs/tasks/D1C_TOOLTIP_PREVIEW_SPECIMEN.md`.


M265 adds a public `Spinner` specimen with a bounded accessible-name field and
a generation-specific identity. Whitespace-only input uses the safe `Loading`
fallback. Native role/name and spoken-announcement checks remain open; see
`docs/tasks/D1C_SPINNER_PREVIEW_SPECIMEN.md`.


M266 adds a public `Divider` specimen with an editable orientation. Short text
children provide visible geometry for both modes. Native orientation and
sizing checks remain open; see
`docs/tasks/D1C_DIVIDER_PREVIEW_SPECIMEN.md`.


M267 adds a public `Skeleton` specimen with a secondary-color switch, fixed
geometry, a progress label, and generation-specific identity. Native visual
and accessibility checks remain open; see
`docs/tasks/D1C_SKELETON_PREVIEW_SPECIMEN.md`.

## M309 — Preview controls layout correction

The compiled-preview specimen controls now use a vertical stack so long control labels remain visible in the Playground's center pane. The first attempt used `.flex_wrap()` on `HStack`; the locked/offline check showed that method is not available, so the row was replaced with `VStack`. The follow-up locked/offline Playground all-targets check passed. The rebuilt macOS 27 Playground visibly showed the List and Table controls in the gallery. This is bounded layout observation only; no general visual acceptance is claimed. Evidence: `target/headstart/evidence/m309-preview-controls-wrap-check.log`, `target/headstart/evidence/m309-preview-controls-wrap-check-followup.log`, `target/headstart/evidence/m309-preview-controls-native-build.log`, and `target/headstart/evidence/m309-preview-controls-native-package.log`.


## M359 — Layout specimen batch (2026-10-21)

The canonical gallery now has signal-free, schema-driven specimens for the
public `Card`, `Container`, and `SplitView` APIs. Their schemas expose bounded
labels, border/surface choices, or orientation; layout identities, children,
panel ranges, and initial sizes stay in compiled factories. All three entries
have empty event and action maps and use the existing descriptor codec. The
focused gallery test module passed 51/51 tests, and the locked/offline
Playground all-targets check passed. No GUI acceptance ran. See
`docs/tasks/D1C_CARD_PREVIEW_SPECIMEN.md`,
`docs/tasks/D1C_CONTAINER_PREVIEW_SPECIMEN.md`, and
`docs/tasks/D1C_SPLITVIEW_PREVIEW_SPECIMEN.md`.

## M361 — PropertyInspector specimen (2026-10-21)

The canonical gallery adds a signal-free, schema-driven specimen for the public
`PropertyInspector`. It exposes three bounded labels and a disabled flag while
keeping editor kinds, row identities, and bindings in the compiled factory.
The focused round-trip test passed 1/1 and locked/offline Playground
all-targets check passed. Native editing and accessibility acceptance remain
open; see `docs/tasks/D1C_PROPERTY_INSPECTOR_PREVIEW_SPECIMEN.md` and the M361
evidence logs listed there.
## M364 component catalog coverage

The source-level catalog audit records a compiled specimen for all 52
implemented public component APIs under `crates/rustui/src/components/`.
`Drawer`, `Outline`, `ResizablePanel`, `GraphView`, and `NodeEditor` remain
scaffold-only modules without implemented APIs and behavior contracts. D1c
does not invent these contracts or add placeholder specimens. See
`docs/tasks/D1C_COMPONENT_CATALOG_COVERAGE.md` for the mapping and specific
contract gaps. This audit does not establish native GUI acceptance.

## M420 — Schema-driven integer inspector values

The generic integer inspector row previously read and wrote the gallery's
shared Stepper state and enforced the Stepper's 0–10 range for every integer
property. This made the Badge specimen's `count` inspector disagree with its
0–999 schema. Integer rows now initialize and read from the current stamped
preview property, and writes use the preview host's schema validation. Only the
Stepper specimen synchronizes the Stepper control state after an accepted
inspector write. The focused regression verifies Badge values 7, 42, and 999,
rejects 1000 without changing the host, and confirms Stepper state updates.

The focused test and locked/offline Playground all-targets check passed. Logs:
`target/headstart/evidence/d1c-integer-inspector-schema-range-m420-20261021.log`
and `target/headstart/evidence/d1c-integer-inspector-schema-range-check-m420-20261021.log`.
Native inspector interaction, focus, accessibility, and screen-reader
acceptance remain open.

## M428 — Boolean inspector reads the mounted property

The generic Boolean property reader now uses the current mounted preview value
when the inspector binding stamp matches. This makes Badge `dot=false` display
the schema value rather than the gallery-level `true` fallback. Stale leases
keep the prior fallback. The focused regression passed 1/1; the file passed
edition-2024 rustfmt and scoped diff checks. Native interaction and
accessibility acceptance remain open. Evidence:
`target/headstart/evidence/d1c-boolean-inspector-mounted-property-20261021.log`.
## M439 — Grid preview specimen

The compiled gallery registers an opt-in `grid-specimen` with bounded `columns`
and `spacing` properties and no preview events. Its focused round-trip test
passed 1/1, covering those properties (`target/headstart/evidence/grid-specimen-m439-test.log`).
This is unit/source evidence; native button activation, visual behavior, and
accessibility acceptance were not exercised.


## M473 — Property-scoped numeric inspector drafts

Generic editable Number rows now keep drafts by `PropertyId` instead of sharing
`amount_draft`. New mounts and successful descriptor imports seed drafts from
the mounted preview properties. Readers reject stale-generation contexts;
writers preserve incomplete drafts, submit finite values to the preview host for
property-specific schema validation, and synchronize shared amount state only
for the Slider, Progress, and amount specimen properties. Slider and dedicated
amount feedback keep the inspector draft map current. The ColorPicker's numeric
channels remain owned by its specialized control and excluded from generic rows,
as specified in `D1C_COLORPICKER_PREVIEW_SPECIMEN.md`.

Focused regression covers Avatar diameter (including 56, above 1), HStack
spacing defaults and edits, visible incomplete input, unchanged owner amount,
old-generation rejection, and imported spacing initialization. Locked/offline
test passed 1/1. Native inspector editing, GUI, accessibility, screen-reader,
and cross-platform acceptance remain open. Evidence:
`target/headstart/evidence/m473-number-inspector-property-drafts-20261021.log`. Locked/offline Playground all-targets check exited 0; evidence: `target/headstart/evidence/m473-playground-all-targets-check-20261021.log`.


## DatePicker inspector canonical-value guard (2026-10-21)

The generic String inspector previously accepted malformed `date` values for the DatePicker specimen because the serialized schema can express only its 10-character bound. `write_text` now applies the specimen’s existing canonical `YYYY-MM-DD` parser before preview-host feedback. A focused regression rejects noncanonical and impossible dates without changing the mounted value, and accepts a canonical date. The locked/offline focused test passed 1/1; scoped rustfmt passed. Native inspector feedback and accessibility remain unverified. Evidence: `target/headstart/evidence/d1c-date-inspector-canonical-write-20261021.log`.

## M495 — stale DatePicker inspector reads

`read_date_value` now requires a matching captured preview stamp and mounted
DatePicker specimen. A deterministic regression captures the prior generation,
imports a different date in a new generation, and confirms the stale binding
returns no value while the current binding reads the new date. Focused test
passed 1/1, compiled-preview module passed 65/65, and locked/offline Playground
all-targets check passed. Evidence: `target/headstart/evidence/m495-stale-date-binding-test-20261021.log`,
`target/headstart/evidence/m495-compiled-preview-module-tests-20261021.log`, and
`target/headstart/evidence/m495-playground-all-targets-check-20261021.log`.

## M498 — stale ColorPicker inspector reads
`read_color_value` now checks the binding stamp and mounted `color-picker-specimen` before reading the preview property. Regression captures the pre-edit stamp, accepts a ColorPicker edit into the next revision, then confirms the retained old context returns `None` and cannot write. Focused test passed 1/1; rustfmt and scoped diff checks passed. Native inspector VoiceOver acceptance remains separate. Evidence: `target/headstart/evidence/m498-color-picker-stale-read-test-20261021.log`.

## M499 — stale SecureField inspector reads
`read_secure_value` now checks the binding stamp and `secure-field-specimen` before returning the owner value. Regression retains the old binding across descriptor import, writes a new-generation secret through a current binding, proves the old binding reads empty and cannot overwrite the new secret, and confirms the descriptor does not contain the value. Focused test passed 1/1; rustfmt and scoped diff checks passed. Native secure-field inspection and VoiceOver remain separate. Evidence: `target/headstart/evidence/m499-secure-field-stale-read-test-20261021.log`.

## M503 — stale compiled-preview text reads

`read_text` now rejects an old preview stamp with an empty value instead of returning the gallery's shared text. The icon accessible-label regression keeps a pre-mount context, mounts the specimen, and verifies the stale reader returns empty while the current reader returns `Search`. The focused icon regression passed 1/1 and the stale-reader suite passed 6/6. Evidence: `target/headstart/evidence/m503-preview-stale-text-reader-20261021.log` and `target/headstart/evidence/m503-preview-stale-read-tests-20261021.log`. Native inspector behavior, VoiceOver, and naturally delayed GPUI callback acceptance remain open.

## M509 — Current text reads for missing properties

For a mounted preview, `read_text` now returns an empty string when the current
component does not expose the requested string property. It retains the
initial owner value only when no preview is mounted and keeps the explicit
placeholder bindings for text inputs. The regression first reproduced stale
text from a previous component; the full compiled-preview Playground library
suite passed 95/95 and the locked/offline all-targets check passed. Evidence:
`target/headstart/evidence/m509-preview-missing-current-property-regression-before-fix-20261021.log`
`target/headstart/evidence/m509-preview-current-missing-property-tests-20261021.log`,
and `target/headstart/evidence/m509-playground-all-targets-check-20261021.log`.
Native inspector interaction and delayed GPUI callbacks remain open.
