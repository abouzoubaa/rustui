# M1286 — Stage 11 Web text-entry and IME boundary

Date: 2026-10-24

## Contract

The X1B accessibility-host acceptance criteria require text entry to preserve
caret and selection through Japanese or another available IME, including
composition, candidate confirmation, and cancellation, with browser and OS
versions recorded. They also state that unit tests for preedit state are
separate from this browser acceptance.

## RustUI source trace

- `TextFieldView` and `TextEditorView` retain GPUI Components input state and
  synchronize controlled owner values from `InputEvent::Change`.
- The single-line path checks `marked_text_range` before replacing retained
  input from an unchanged owner value. An external owner update remains
  authoritative during marked text.
- The multiline path tracks marked-text state and restores an unchanged owner
  value after a canceled preedit. Existing focused unit cases cover active
  preedit, cancellation, external owner updates, and ordinary local edits.
- These are local state-reconciliation paths; they do not implement browser
  IME composition or prove caret/selection behavior.

## Web ownership boundary and result

The pinned GPUI Web backend owns a hidden textarea IME mirror and routes
browser input/composition through GPUI's input-handler protocol. Its event
timing, browser/OS candidate UI, caret/selection behavior, and confirmation or
cancellation are runtime behaviors. The X1B contract provides no separate
RustUI-only mechanism for those behaviors, and this source review identifies
no contract-backed local defect to change.

No RustUI, vendored, or dependency source changed for M1286. No browser/OS IME
test was run; composition, candidates, caret/selection, and full Stage 11
acceptance remain unverified. A browser run using an available IME with browser
and OS versions recorded remains necessary. Keep the implementation goal open.

## Web bundle source-binding support (2026-10-24)

`scripts/build-playground-web.sh` now writes `bundle-manifest.json` into each
published bundle. The manifest records the Git commit, a deterministic
SHA-256 over Cargo/build inputs and files under `crates/` and `apps/`, and
SHA-256 hashes for every packaged file. The builder snapshots sources before
Cargo runs and checks them again before publishing; source drift rejects the
new bundle and preserves the previous one.

The focused fixture check passed with `sh scripts/tests/test_build_playground_web.sh`.
It verified source and artifact hashes and that source changes during a build
are rejected. No real Wasm bundle was rebuilt and no browser or IME run was
performed, so the existing served bundle is still unbound until rebuilt and
its manifest and artifact hashes are recorded.

## Real Web build attempt and existing-host smoke (2026-10-24)

The first `./scripts/build-playground-web.sh` attempt stopped before
compilation because `vendor/cc-1.8.0/src/target/llvm.rs` was missing. I restored
only that file byte-for-byte from another worktree after verifying its SHA-256
against this worktree's checksum manifest. The retry advanced to the next
missing vendored file, `vendor/cc-1.8.0/src/target/generated.rs`, and stopped
before compilation. No other vendor files were restored or changed. The
existing bundle was preserved and still has no `bundle-manifest.json` (the
running server returned HTTP 404). Its served HTML, JavaScript, and Wasm
SHA-256 values were
`514cc121…b3a838`, `42375a5b…fef7801`, and `46e4e153…ef964be` respectively.

The existing Chrome page at `http://127.0.0.1:8766/` loaded. A `button` search
showed three catalog rows; selecting `Français` visibly changed labels to
`Langues` and `Fermer`. Search and locale were restored to `text` and English.
The Text story still displayed Chinese glyph boxes. A current-source browser
smoke could not run because the build produced no new bundle. This is only a
smoke check of the existing, unbound bundle; it does not establish current-
source browser, accessibility-tree, CJK font, or IME acceptance.

## Current-source Web build and browser smoke (2026-10-24)

After the first retry stopped on `generated.rs`, I restored that file
byte-for-byte from another worktree. Its SHA-256
`726791d94043132ad1a04a4eba3a2638db04e6eb52245a0ff6d1cc419b3d0a8b` matched
the existing `.cargo-checksum.json`; `llvm.rs` had already been restored with
the same exact-manifest check. No vendored file contents or checksum metadata
were edited.

The next `./scripts/build-playground-web.sh` completed successfully in the
optimized release profile and wrote `target/headstart/playground-web`. The
source manifest binds commit `1cb2e4996b1b5cafba2d5823cd31cf78be7e8e44` and
source digest `5602af766d067aa0e39dea71e4473424ee7eb1c116aa2377c97b75e68dc7a481`;
it records hashes for 39 packaged files. The existing server on
`127.0.0.1:8766` returned HTTP 200 for `bundle-manifest.json`, byte-identical
to the manifest in the build output.

In a browser page served by that host, the RustUI Playground rendered. Selecting
French visibly changed the label `Locales` to `Langues`; selecting English
restored `Locales`. The browser accessibility tree exposed the page, skip link,
Playground container, and one active text-entry area, but not the component
rows or controls. This establishes current-source build and bounded locale
interaction only. It does not establish search filtering, keyboard
accessibility, browser accessibility-tree coverage, font/glyph coverage, or
text-entry/IME behavior. No GUI test application was launched or closed; the
existing local server and other running applications were left untouched.

## Focused Web TextField IME run sheet (planned; not executed)

This run is limited to the Playground catalog search field and a real
browser/OS IME session. It is separate from general Stage 11 checks for
keyboard navigation, references, fonts, locale, and full accessibility.

### Prerequisites

- Use an existing hash-bound Playground Web bundle and record the bundle
  identity. If the available bundle cannot be tied to its sources, record the
  prerequisite as unmet; this M1286 run does not rebuild it.
- Use Google Chrome stable on macOS with a hardware keyboard. Record the full
  Chrome version and macOS version/build.
- Select Apple's Japanese input source in Romaji mode with Hiragana
  composition. Record the exact input-source and keyboard-layout names shown
  by macOS. Confirm the Japanese input source is active before typing.
- Open a fresh Playground page with an empty search query and focus the
  catalog search field.

The previous browser follow-up's `current_input_source_romaji_probe` typed
`nihon` and Space but retained the literal value `nihon `; it explicitly did
not change the system input source or exercise composition. Do not count that
probe as an IME result.

### Sequence and expected observations

1. Type `ab` and press Left once. Confirm the field shows `ab` with the caret
   between the letters.
2. With Japanese Romaji active, type `nihongo`. Observe and record the marked
   preedit, candidate UI, caret/selection, and any transient query/filter
   changes. Do not require a particular transient filter result; the field
   should retain the marked text through rerenders.
3. Choose the visible `日本語` candidate and confirm it. Record the key
   sequence used. The committed field and owner-bound query should be
   `a日本語b`, without duplicated or lost characters; results should reflect
   the committed query.
4. Move to the end of the field and press Left once, placing the caret before
   `b`. Type `arigatou` to start another composition, then press Escape to
   cancel it. If Escape only closes the candidate window, press it again and
   record that. The value should return to `a日本語b` with no canceled text
   left behind.
5. Press Backspace at the same caret position. The value should become
   `a日本b`, deleting the preceding `語` and leaving the caret at the edit.
6. Start `nihongo` composition again and press Backspace while it is marked.
   Record how the active preedit/candidate changes and confirm the field
   remains usable. Cancel with Escape as above; the committed value should
   remain `a日本b`.

At each step record visible field value, caret/selection, whether marked text
remains active, and catalog-result behavior. After commit, cancellation, and
deletion, verify the controlled RustUI value agrees with the committed visible
text and does not revert on rerender.

If a browser accessibility snapshot is available, capture the focused
text-entry role and value before composition, during preedit if exposed, and
after commit/cancel/delete. Record observed focus and value. The M1285
accessible-name gap is a separate gate; a missing name is not an M1286
failure. Do not count the OS candidate UI as a browser accessibility node
unless the snapshot actually exposes it.

### Acceptance boundary

Pass this focused run only if composition commits without lost or duplicated
text, cancellation restores the pre-composition value, deletion changes the
intended preceding character, the controlled query follows committed edits,
and the field remains usable with the expected caret/selection behavior.
Attach the browser, OS, input-source, bundle-identity, and observation record.

Without a real browser session and installed OS IME, scripted or unit-level
checks cannot establish browser event ordering, candidate UI behavior,
composition confirmation/cancellation, or actual caret/selection preservation.
The existing RustUI preedit tests cover owner-value reconciliation only and
do not satisfy this browser/OS acceptance.
