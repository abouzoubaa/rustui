# Framework tools review (T9R)

Review of the current native source and tests. No product code, public API,
capability status, roadmap, or verification scripts changed. Physical line counts
include inline tests and comments; shared files are identified rather than counted
as separate implementations. PASS below applies to the stated focused path, not
unmeasured visual, accessibility, or performance acceptance. No failing test or
compiler error was observed in the focused runs.

| Tool | Works? | Tests exercised now | Used by | Recommendation |
| --- | --- | --- | --- | --- |
| R22 descriptor/schema/codec | PASS: validate and round-trip declared inputs | 34 core tests | PreviewHost, CompositionHost, Playground registry/import/export | KEEP |
| R23 compiled preview/inspector | PASS: property feedback → export → reimport; initial story draw | 4 host + 103 gallery; shared draw smoke | Playground Layout story; component specimens | KEEP |
| R24 structural composition editing | PASS: add/move/remove/rename → compiled factory; codec/remount | 16 core + 4 story; shared draw smoke | Playground Layout story; deferred reference adapter | KEEP |
| R25 external consumer adapter | deferred platform; end-to-end NOT VERIFIED | 9 native protocol fixtures pass in global check; 10 JS fixtures not run | Playground reference host/client; not the standalone external-consumer example | DEFER |
| R26 live theme/data reload | PASS: explicit file reload and invalid-update recovery; automatic dispatched path NOT VERIFIED | 10 file + 3 ownership + 3 inspector; shared draw smoke | Playground Divider/Theme Layout demo; ThemeFile public API | FIX |

The shared smoke is one test, not three additional tests. The seven focused
commands exercised **178 distinct tests: 178 passed, 0 failed, 0 ignored**.
The required workspace check also exercises native protocol fixtures; that is
source coverage, not execution or acceptance of R25's deferred adapter path.

## R22 — descriptor/schema/codec validation

Validates versioned component catalogs, property values, defaults, and allowlisted
action references. Encodes/decodes bounded JSON descriptors without serializing
Rust views, callbacks, entities, or private owner state.

Public entry points under `rustui::preview`: `ComponentCatalog`, `ComponentSchema`,
`PropertySchema`, `PropertyKind`, `PropertyValue`, `EventSchema`, `PreviewDocument`,
`ValidatedPreview`, `PreviewError`, and typed IDs (`ComponentId`, `PreviewId`,
`PropertyId`, `EventId`, `ActionRef`). Operations include
`ComponentCatalog::{new,register_component,register_action}`,
`PreviewDocument::new`, `ValidatedPreview::{apply_batch,apply_batch_with}`,
`validate_catalog`, `validate_preview`, `encode_catalog`, `decode_catalog`,
`encode_preview`, `decode_preview`, and `decode_property_value`.
`PreviewRegistry::{new,register,register_action,catalog}` connects these contracts
to compiled factories; `register_composable` is shared with R24.

Measurements:

```sh
wc -l crates/rustui/src/preview/*.rs
```

```text
 953 codec.rs
1293 composition.rs
 388 host.rs
  30 mod.rs
 335 model.rs
 676 registry.rs
 799 schema.rs
4474 total
```

Paths above share `crates/rustui/src/preview/`. R22's primary files are
`model.rs`, `schema.rs`, `codec.rs`, `registry.rs`: **2,763 lines**.
`mod.rs` is the shared public export surface; `host.rs` and `composition.rs`
are consumers reviewed below.

Test-file counts, measured with
`rg -c '#\[(test|gpui::test)\]' crates/rustui/src/preview/*.rs`:

```text
codec.rs:14
schema.rs:10
registry.rs:10
host.rs:4
composition.rs:16
```

```sh
rustup run 1.97.1 cargo test -p rustui --lib preview:: --locked --offline
```

```text
preview_exit=0
54 passed; 0 failed; 0 ignored
```

The 54 comprise R22's 34, R23's 4, and R24's 16. **End-to-end PASS** for
`preview::codec::tests::catalog_and_preview_round_trip_deterministically`:
registered catalog → encoded catalog → decoded catalog → validated preview →
encoded preview → decoded preview, asserting exact equality and deterministic
re-encoding. Other passing cases reject duplicate keys, malformed action
references, unsupported versions, oversized documents, lossy numeric tokens,
invalid defaults, and invalid batches without poisoning registration.

Usage command and output:

```sh
rg -l 'validate_preview|decode_preview|PreviewRegistry' \
  crates/rustui/src/preview/{host,composition}.rs \
  apps/rustui-playground/src/stories/compiled_preview_gallery/{mod,registry}.rs
```

```text
apps/rustui-playground/src/stories/compiled_preview_gallery/registry.rs
crates/rustui/src/preview/host.rs
crates/rustui/src/preview/composition.rs
```

The gallery imports the public module and uses the codecs in its split files.
README describes descriptor-first composition; archived design notes include
`docs/archive/tasks/D1B_COMPILED_PREVIEW_HOST.md`. The standalone external-consumer
example does not use these APIs (reference search recorded in R25). This is used
framework infrastructure, not dead code.

Old audit row R22 (in `docs/archive/tasks/ROADMAP_REMAINING_AUDIT.md:4440`)
said the source/tests existed but full contract review remained. Current tests
and inspection confirm bounded inputs, exact integer handling, transactional
validation, and factory registration; exhaustive contract/security analysis is
still NOT VERIFIED. **Recommendation: KEEP** for its declared descriptor contract.

## R23 — compiled preview and schema-driven inspector

Mounts explicitly registered compiled RustUI factories and exposes editable
schema properties through the Playground's PropertyInspector. Generation/revision
stamps reject stale writes/actions; import/export preserves the descriptor boundary.

Public library entry points: `PreviewRegistry`,
`PreviewHost::{new,catalog,snapshot,mount,teardown,edit,binding_feedback,dispatch,render,render_factory}`,
`PreviewStamp`, `PreviewSnapshot`, and `PreviewSignal` (including
`PreviewSnapshot::signal`). The Playground implementation and its `demo` function
are app-local, not a separately exported inspector SDK.

Source measurements:

```sh
wc -l apps/rustui-playground/src/stories/compiled_preview_gallery/{mod,registry,gallery}.rs \
  apps/rustui-playground/src/stories/compiled_preview_gallery/gallery/*.rs
```

```text
 338 mod.rs
  52 registry.rs
  19 gallery.rs
 853 gallery/basic.rs
1117 gallery/bindings.rs
  83 gallery/canvas.rs
 575 gallery/data.rs
 856 gallery/feedback.rs
  25 gallery/graph.rs
 703 gallery/layout.rs
1301 gallery/mount.rs
 618 gallery/overlay.rs
 928 gallery/text.rs
  82 gallery/transport.rs
 695 gallery/view.rs
8245 total
```

Together with shared `crates/rustui/src/preview/host.rs` (388 lines), this is
**8,633 source-file lines**, before the gallery's separate test directory.
The registry/schema/codec implementation is shared with R22.

Separate test files under the gallery's `tests/`, measured with the same
`rg -c` expression:

| File | Lines | Tests |
| --- | ---: | ---: |
| suite_01.rs | 987 | 23 |
| suite_02.rs | 995 | 15 |
| suite_03.rs | 1028 | 16 |
| suite_04.rs | 1025 | 13 |
| suite_05.rs | 1021 | 12 |
| suite_06.rs | 1029 | 18 |
| suite_07.rs | 295 | 5 |
| graph.rs | 16 | 1 |
| mod.rs | 14 | 0 (includes the suites) |
| Total | 6410 | 103 |

```sh
rustup run 1.97.1 cargo test -p rustui-playground --lib compiled_preview_gallery --locked --offline
```

```text
gallery_exit=0
103 passed; 0 failed; 0 ignored
```

The four core host tests passed in R22's 54-test command. **End-to-end PASS** for
`date_picker_specimen_round_trips_optional_dates_and_rejects_stale_feedback`:
mount registered specimen → write selected date through the inspector binding
path → export → decode → import/remount → assert selected date survives and stale
feedback is rejected. Malformed dates preserve the current mount. The schema
amount-editor test also verifies that editing the inspector updates its preview
owner. These are actual application model/host/codec paths; they do not simulate
pointer or keyboard input into every inspector row.

```sh
rustup run 1.97.1 cargo test -p rustui-playground --lib story_smoke_tests --locked --offline
```

```text
story_smoke_exit=0
story_smoke_tests::every_registered_story_mounts_draws_and_exposes_its_title ... ok
1 passed; 0 failed; 0 ignored
```

This separately mounts/draws every registered Playground story in GPUI test
windows, including Layout. It establishes initial story rendering, not a complete
UI-driven edit/export/import sequence or native pixel fidelity.

Usage command (also used for R24):

```sh
rg -n 'compiled_preview_gallery::demo|composition_gallery::demo|foundation_additions::foundation_additions' \
  apps/rustui-playground/src/stories/mod.rs
```

```text
115: "divider" => Some(AnyView::new(foundation_additions::foundation_additions())),
869: (selected == StoryId::Layout && self.w1_step() == 0).then(compiled_preview_gallery::demo)
870: (selected == StoryId::Layout && self.w1_step() == 0).then(composition_gallery::demo)
```

The current fixture asserts **78 implemented component specimen IDs**. R25 also
reuses the same factories. Current docs reference preview specimens in
`docs/COMPONENT_CATALOG.md`; the detailed inspector contract is archived at
`docs/archive/tasks/D1C_PLAYGROUND_PREVIEW_INSPECTOR.md`. No standalone
external-consumer example usage was found. It is not dead code.

Old audit row R23 (`:4441`) recorded historical native edit/export/remount
observations and left broader acceptance open. The current focused round trip
and initial draw pass; full dispatched UI editing and visual acceptance remain
NOT VERIFIED in this ticket. **Recommendation: KEEP** the tested local tool.

## R24 — structural composition editing

Edits a versioned, stable-ID graph of registered compiled components and ordered
slots. Insert, remove, move, and property batches validate atomically before the
host recursively creates ordinary RustUI views.

Public entry points: `CompositionDocument`, `CompositionNode`, `CompositionSlots`,
`CompositionEdit`, `NodeId`, `SlotId`, `SlotSchema`, `CompositionStamp`,
`CompositionNodeSnapshot`, `CompositionSignal`, composition format/limit constants,
`PreviewRegistry::{register_composable,composition_slots}`,
`encode_composition`, `decode_composition`, and
`CompositionHost::{new,catalog,composition_slots,mount,mount_source,document,stamp,unmount,encode,snapshot,dispatch,edit,render}`.

```sh
wc -l crates/rustui/src/preview/composition.rs \
  apps/rustui-playground/src/stories/composition_gallery.rs
```

```text
1293 crates/rustui/src/preview/composition.rs
 519 apps/rustui-playground/src/stories/composition_gallery.rs
1812 total
```

`registry.rs` (676 lines) and the R22 model/schema/codec are shared support.
There are **16 inline core tests** and **4 inline story tests**, plus the shared
story draw smoke. The 16 core tests passed in the `preview::` command above.

```sh
rustup run 1.97.1 cargo test -p rustui-playground --lib composition_gallery --locked --offline
```

```text
composition_exit=0
public_factory_composition_add_move_remove_and_rejection_are_live ... ok
4 passed; 0 failed; 0 ignored
```

**End-to-end PASS** for the named test: construct the public registry and mounted
graph → add child → reorder → remove → rename property → call the compiled render
factory → reject excess children without advancing the stamp. Core
`recursive_composable_render_and_codec_keep_leaf_api_separate` passes the graph
codec round trip and recursive factory path. Core
`signals_are_node_scoped_and_rejected_after_edit_unmount_or_remount` also exports,
rejects invalid source without replacing the document, edits/unmounts, and remounts
exported source while rejecting obsolete signals. These paths were run now.
The shared GPUI smoke separately confirms the default Layout story draws.

Usage: `stories/mod.rs:870` mounts the native Layout demo (command/output in R23).
R25 delegates structural requests to the same `CompositionHost`; RustUI itself
has no dependency on that adapter. README describes compiled composition editing;
`docs/archive/tasks/STAGE9_DESCRIPTOR_COMPOSITION_IMPLEMENTATION.md` documents its
source contract. The standalone external-consumer example does not use it.

Old row R24 (`:4442`) described the graph/slot/atomic-edit implementation and
left broader native/performance acceptance open. That implementation remains
present; current 20 focused tests and the shared initial-draw smoke pass.
UI-button dispatch across the complete sequence and performance at graph limits
remain NOT VERIFIED. **Recommendation: KEEP** the bounded compiled composition API.

## R25 — external consumer adapter

Adapts the existing preview/composition hosts to an explicit consumer protocol,
with sessions, exact stamps, strict bounded messages, and an owner action policy.
It is a consumer-owned reference adapter, not a general RustUI state-export API.

**deferred platform**. No targeted adapter run or end-to-end acceptance was
performed. There is no observed failing assertion to report for this skipped path.

There is no public RustUI adapter entry point. App-local Rust types include
`Stamp`, `Request`, `Reply`, `Host`, `Session::{handle,close}`, `Outcome`, and
`Bridge::{new,install,bind_owner,close}`; their containing types/modules are private
or `pub(crate)`. The consumer-facing module exports `ReferenceClient` with
`connect`, `send`, `receive`, `disconnect`, and `dispose`; transport helpers include
`requestedProfile`, `freshToken`, `createBridge`, `markReady`, `postReply`, and
`closeBridge`.

Source sizes measured by `wc -l`:

| File | Lines |
| --- | ---: |
| apps/rustui-playground/src/preview_transport.rs | 1241 |
| apps/rustui-playground/src/preview_transport_web.rs | 204 |
| apps/rustui-playground/web/preview-transport.js | 20 |
| apps/rustui-playground/web/reference-client.js | 89 |
| apps/rustui-playground/web/reference-host.js | 94 |
| apps/rustui-playground/web/reference-host.html | 20 |
| gallery/transport.rs (under R23's gallery directory) | 82 |

Reference entry functions in both gallery modules and platform-gated wiring in
`apps/rustui-playground/src/lib.rs` are shared support. Test inventory:
`preview_transport.rs` contains **9 Rust protocol fixtures**;
`apps/rustui-playground/web/tests/preview-transport.test.mjs` is **162 lines,
10 JS fixtures** (measured with `rg -c '^test\('`). The packaging fixture
`scripts/tests/test_build_playground_web.sh` references these assets; it was not run.
These inventories are not end-to-end pass claims. The required native workspace
check passed all nine Rust fixtures but did not exercise the deferred adapter.
Its output includes `preview_transport::tests::generic_catalog_mount_edit_export_and_replays_use_existing_host ... ok`;
the other eight listed native protocol cases also reported `ok`.

Usage commands and outputs:

```sh
rg -n 'reference-client|ReferenceClient' apps/rustui-playground/web/reference-host.js
```

```text
1:import { ReferenceClient } from './reference-client.js';
30:  client = new ReferenceClient(frame, $('profile').value);
```

```sh
rg -n 'PreviewRegistry|PreviewHost|CompositionHost|ThemeFile' examples/external-consumer/src/main.rs
```

```text
external_tools_reference_exit=1
(no matches)
```

That example is a Counter/name-field Git consumer, not this adapter. Native
preview/structural editing use R22–R24 directly; they do not need R25.
README still says external preview-host integration/release acceptance is
incomplete. The named reference contract lives in
`docs/archive/tasks/D1D_BROWSER_PREVIEW_HOST.md`. The adapter has a reference
consumer, so lack of standalone-example usage does not make it dead code.

Old row R25 (`:4443`) reported implemented reference contracts and historical
bounded round trips while leaving broader platform acceptance open. Those old
results are not current verification. **Recommendation: DEFER** until the owner
reopens its target platform and decides whether to support that named reference
consumer or a different external application. No deletion is recommended now.

## R26 — live theme/data reload

`ThemeFile` reads and validates JSON theme data, then transactionally activates it
in a GPUI app. Playground adds manual actions and an opt-in 350 ms polling loop;
this is theme-data reload, not a general data loader or Rust source hot reload.

Public entry points: `rustui::theme::{ThemeFile,ThemeFileError}` with
`ThemeFile::{open,theme,reload,reload_if_changed}`. Shared activation/inspection
uses `parse_theme`, `load_theme`, `serialize_theme`, `activate_theme`, and
`theme::registry::{active_theme,theme_coverage}`. `ReloadState`, `ReloadLease`,
`FoundationAdditions::{reload_action,toggle_auto_action}`, and the theme inspector
are private Playground implementation details. `ThemeFile` explicitly does not
create a watcher; the caller chooses when to reload. The native Playground
uses `RUSTUI_THEME_RELOAD_PATH` when set, otherwise its resolved
`themes/macos27.json` resource path.

```sh
wc -l crates/rustui/src/theme/watcher.rs \
  apps/rustui-playground/src/stories/{foundation_additions,theme_inspector}.rs
```

```text
 559 crates/rustui/src/theme/watcher.rs
 710 apps/rustui-playground/src/stories/foundation_additions.rs
 353 apps/rustui-playground/src/stories/theme_inspector.rs
1622 total
```

Shared theme support, measured with `wc -l crates/rustui/src/theme/*.rs`:
`invalidation.rs` 13, `loader.rs` 367, `mod.rs` 25, `registry.rs` 999,
`resolver.rs` 1003, `schema.rs` 2450, `tokens.rs` 360; the whole theme directory
including `watcher.rs` is 5,776 lines. These files also support ordinary theming.

Test files/counts: `watcher.rs` **10** (three GPUI app tests and seven unit tests),
`foundation_additions.rs` **3**, `theme_inspector.rs` **3**, plus the shared initial
draw smoke. Shared theme parsing/resolution/registry tests also run in check.sh;
they are not counted as additional reload-path tests here.

```sh
rustup run 1.97.1 cargo test -p rustui --lib theme::watcher:: --locked --offline
rustup run 1.97.1 cargo test -p rustui-playground --lib foundation_additions --locked --offline
rustup run 1.97.1 cargo test -p rustui-playground --lib theme_inspector --locked --offline
```

```text
watcher_exit=0; 10 passed; 0 failed; 0 ignored
foundation_exit=0; 3 passed; 0 failed; 0 ignored
inspector_exit=0; 3 passed; 0 failed; 0 ignored
```

**End-to-end PASS for explicit reload**:
`theme::watcher::tests::file_reload_recovers_after_invalid_update_in_app` writes a
real temporary theme file, calls the public reload method in a GPUI app, rejects
invalid JSON without changing the active revision/document, repairs the file,
and observes an active revision increment for changed valid source. The final
change is trailing whitespace, not a changed rendered color. The forced-reload
test separately restores Light after another activation selected Dark.

**Automatic dispatched path NOT VERIFIED**: no existing focused test dispatches
`toggle_auto_action`, advances its polling timer, changes a theme token, and
asserts a mounted app updates. The only watcher lifecycle test,
`reload_actions_do_not_keep_owner_alive_or_survive_unmount`, constructs the actions
and checks weak leases/strong counts without dispatching either action. This is
a coverage gap, not an observed runtime failure; there is no failing assertion
or error to quote. The initial-draw smoke does not enable automatic reload.

Usage: `stories/mod.rs:115` maps Divider to the live-theme/layout demo, which also
mounts `theme_inspector::demo`. `foundation_additions.rs:169,313` opens ThemeFile.
`docs/PUBLIC_API.md:1515` describes transactional ThemeFile replacement; archived
R26 notes contain the earlier reload investigation. The standalone
external-consumer example only parses/activates its bundled theme; it does not
reload. The tool is used, not dead code.

Old row R26 (`:4444`) said ownership/unit checks did not establish dispatched
native event-path acceptance and invalid-update recovery remained open. Recovery
is now verified for the explicit public API and inspector state. The dispatched
automatic path and actual changed-token rendering remain open.

**Recommendation: FIX**, as two independently bounded follow-up tickets:

1. **At most 2 hours:** add a deterministic GPUI-window test dispatching the
   Playground auto action against a temporary theme file; advance executor time,
   change a visible token, and assert both ActiveTheme and a mounted view update.
   Stop the watcher and prove later file edits do not activate a theme.
2. **At most 2 hours:** extend that dispatched path with invalid JSON → preserved
   last-good theme/error → repaired valid data/error cleared, then remove the
   owner while polling and verify no task/owner retention or later activation.

These are test/verification repairs; this review found no justified production
fix or need to add a generic reload API.

## Required check

```sh
scripts/check.sh
```

```text
check_exit=0
1421 passed; 0 failed; 4 ignored  (rustui)
   9 passed; 0 failed; 0 ignored  (rustui-core)
  86 passed; 0 failed; 0 ignored  (rustui-i18n)
  40 passed; 0 failed; 0 ignored  (rustui-macros)
 253 passed; 0 failed; 0 ignored  (rustui-playground)
   0 passed; 0 failed; 0 ignored  (rustui-rgpe)
```

Total: **1,809 passed, 0 failed, 4 existing ignored**. Formatting and the locked,
offline workspace all-target check also exited 0 through this script.
No tests were added, removed, changed, or newly ignored for this review.

Before merging, `./scripts/verify-release.sh source` also exited 0;
its script-test suite reported `Ran 644 tests` and `OK`.
