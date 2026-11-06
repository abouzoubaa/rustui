# M1285 — Stage 11 Web search accessible-name package

Date: 2026-10-24

## Decision requested

Authorize authoring and reviewing the narrow, versioned same-version patch
described below, after the approved M855 v3 keyboard/focus patch. This package
does not authorize applying the proposed patch, changing dependency revisions,
building, or claiming browser or VoiceOver acceptance. Those actions remain
separate gates.

## Observed issue

The supplied current-bundle observation reported that the focused Playground
search entry had no accessible name, despite its visible placeholder. This
turn did not rerun the browser or bind that observation to a fresh bundle hash.

## Source trace

- `apps/rustui-playground/src/lib.rs` creates a RustUI `TextField` with the
  placeholder `Search N components`, query binding, and stable ID
  `playground-search`.
- `crates/rustui/src/components/input/text_field.rs` forwards the optional
  accessibility label. In the pinned GPUI Components input renderer, the
  non-mask placeholder is also used as the default `aria_label`; it sets the
  AccessKit text-input role, value, and placeholder (`vendor/gpui-component-0.7.3/src/input/input.rs`).
  AccessKit explicitly distinguishes `Node::label()` from
  `Node::placeholder()`; this control's renderer supplies the former from the
  placeholder fallback.
- GPUI builds frame tree updates in
  `vendor/gpui-pre-0.3.9/src/window/a11y.rs` and submits them at
  `vendor/gpui-pre-0.3.9/src/window.rs:3795`. However, its
  `A11yCallbacks` initialization is guarded by `cfg(not(target_family =
  "wasm"))` near line 1669. The active flag therefore remains false on WASM,
  so the frame path does not construct or submit updates. Action dispatch is
  also non-WASM-only near line 7033.
- The platform trait's accessibility methods default to no-ops, and pinned
  `gpui-pre-web` does not override them. Thus a Web-host-only patch cannot
  receive a tree under the current WASM core configuration.
- Web already maintains one focused hidden IME `<textarea>` in
  `vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs`. Playground's
  `web/index.html` mutation observer gives it a generic label when inserted;
  the observer does not watch attribute changes. Reusing this element avoids
  a duplicate textbox and keeps its existing value, selection, focus, and IME
  input path authoritative.

## Narrow v1 patch package

Prepare one hash-checked patch package for the pinned `gpui-pre = 0.3.9` and
`gpui-pre-web = 0.3.9` pair. Keep package versions and registry sources fixed;
apply and select it only through the established M855 same-version local
override process. The proposal is at
`patches/proposed/gpui-pre-web-accessible-name-v1.patch`; its staged pre/post
image checker is `scripts/apply_gpui_web_accessible_name_patch.py`.

1. In GPUI core, enable the existing AccessKit frame-tree production path on
   WASM unless the application was created with `Application::new_inaccessible`.
   Keep the existing native activation and action-callback behavior unchanged.
   WASM does not install the native callbacks; action-request dispatch is
   explicitly outside this label-only slice.
2. In `gpui-pre-web`, eagerly keep the semantic tree current because browsers
   provide no reliable signal that a screen reader is attached. Consume each
   `TreeUpdate`, locate its focused node, and, when it is `Role::TextInput`,
   set the existing IME textarea's `aria-label` from `Node::label()`; remove
   the generic shell label when there is no focused text-input label. Do not
   write the textarea value, selection, or focus from the tree. Its existing
   IME mirror remains the owner of those states.
3. Make no RustUI API, Playground control, new dependency, or package-revision
   change. Keep this projection scoped to the single focused text-input
   representation. It does not expose every AccessKit node, route AccessKit
   actions, or complete X1B accessibility acceptance.

This plan reuses GPUI's tree and the existing DOM input. It avoids a separate
DOM control while respecting the current Web IME ownership boundary. The
semantic tree must stay active on Web, so the full tree-build cost is a known
runtime cost and needs measurement in later acceptance.

## Baseline identity

Pinned upstream snapshot: Zed revision
`93d745668805029476e6d7fbb19652116fed51de`; Components `0.7.3`.

| Input | Identity at package preparation |
| --- | --- |
| `gpui-pre 0.3.9` registry checksum | `34f61c1cfd36fc9c436af09308c963822ae27a45a84feb21a77ba5c9e051f1c0` |
| Vendored GPUI checksum manifest | `7b8e889ae4ac0cbe651b654f5e39d9003969d09ae4a70aff4147f521eda8faa0` |
| `vendor/gpui-pre-0.3.9/src/window.rs` SHA-256 | `44f32319b9385fa6dbc15c36b3edea00a26208309b37283ae4f9bd4bead674f8` |
| `gpui-pre-web 0.3.9` registry checksum | `aedd55b99dc77e28df8202b0ea868aa5c90fda4aefff539ea21087d7c2ff4bfb` |
| Vendored Web checksum manifest | `aad9503f554e462d582461d008ea4bc0dda00021812ccf6b876734b55757579e` |
| `vendor/gpui-pre-web-0.3.9/src/window.rs` SHA-256 | `3779ba8d5647009b80d0ea52dc7823806640741d3a1cfa9a0a7752ec35fe7dd8` |
| `vendor/gpui-pre-web-0.3.9/src/ime_mirror.rs` SHA-256 | `4061ebd319b5928c2a25f659c4f1db9adb193d5873bbbfeba36468ca2706895d` |
| AccessKit `0.24.1` lock checksum | `d3b7f7f85a7e5f68090000ed7622545829afd484d210358702ae4cb97dd0c320` |
| GPUI Components `0.7.3` registry checksum | `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9` |

Recheck all source and package identities against the selected checkout before
creating or applying a patch. The Web source baseline above includes M855 v3.

### Candidate package review — 2026-10-24

- Patch SHA-256:
  `8fbb44e848e423978c7a216dcd3e814fdf477e520657dba7a7a8bd8653ed9881`.
- Exact staged postimages:
  - GPUI core `src/window.rs`:
    `66d78f6ca924808cba8a9121fd6e44e6537ccf87865c6b443b11a4fe1c140855`
  - GPUI Web `src/window.rs`:
    `07f241ba3476ce5b3e76927e768a00a84207869115482b7dba1297971d9d4bfa`
  - GPUI Web `src/ime_mirror.rs`:
    `5647cf3f467837f1b1747aac15fb0f4e6bac1bbe862d040427ba9e48b841088f`
- `python3 scripts/apply_gpui_web_accessible_name_patch.py --check` passed.
  It verified package versions, pristine checksum manifests, the candidate
  digest, all source preimages, exact patch application without fuzz/offset,
  and all three staged postimage hashes. M855's independent `--check` also
  passed at HEAD `1cb2e4996b1b5cafba2d5823cd31cf78be7e8e44`.
- Review confirms the core change leaves the native callback/action path
  unchanged and `Application::new_inaccessible` disabled. The Web callback
  selects the focused node from GPUI's full per-frame `TreeUpdate`, projects a
  nonempty label only for `Role::TextInput`, and removes the generic label for
  every other state. The IME method changes only the `aria-label` attribute.
- Applying this candidate requires separate approval to select the local
  `gpui-pre 0.3.9` package in `Cargo.toml`. The helper does not edit Cargo
  configuration. No vendored source was changed, and no build or browser
  acceptance was run.

## Review and acceptance oracle

Review the generated diff against those exact inputs. Confirm the inaccessible
application path stays disabled, WASM tree production yields the focused input
node, a missing label removes the generic textarea name, and tree updates
never modify IME value or selection. Confirm action dispatch remains an
explicit limitation rather than silently implying full AccessKit support.

After separate approval to apply the patch, use a fresh, hash-bound Web bundle
in current Chrome on macOS. Capture the browser/OS versions and accessibility
snapshot. The focused search entry should appear exactly once as a textbox
named `Search N components`; its current query should appear as its value.
Typing must filter the catalog while the same textbox remains named and
focused. With VoiceOver enabled, verify it announces the expected name, role,
and value. Repeat M855 keyboard focus and Space-activation checks. This
acceptance has not run and must not be reported as passed.

## Limits and status

The supplied observation is not a fresh, hash-bound browser result. The
hash-checked candidate was authored and reviewed but not applied. Browser AX
behavior after the candidate, VoiceOver speech, WASM compilation, and
tree-build cost remain unverified. This narrow search-name slice does not
close X1B or Stage 11; full control semantics, screen-reader actions, IME
behavior, and broader browser/platform coverage remain separate work.

## Browser accessibility-tree baseline before M1285 patch

Date: 2026-10-24

Observed the existing packaged Playground in Google Chrome on macOS, with the
search field focused and empty, at `http://127.0.0.1:8766/`. The Chrome app
accessibility snapshot exposed:

```text
HTML content RustUI Playground
  container RustUI Component Playground
  text entry area (settable) RustUI active text input
```

The focused text-entry node had the generic `RustUI active text input` name;
the snapshot did not expose `Search N components` or a query value. This
confirms the pre-patch browser-tree symptom for this observed bundle. Chrome
DevTools MCP could not attach because its managed Chrome profile was already
running, so this is the native Chrome application accessibility snapshot.

### Runtime and bundle identity

- Browser: Google Chrome `155.0.8059.86`.
- OS: macOS `27.0`, build `26A5425a`.
- Bundle root: `target/headstart/playground-web`.
- `index.html` SHA-256: `514cc121b15e4195b66bea3fef5478dd3103a8b3569c2f30b9de21e689b3a838`.
- `rustui_playground.js` SHA-256: `42375a5b51056a86fc89b9c0e898d4520f3b96f68950e217f4d5d86f9fef7801`.
- `rustui_playground_bg.wasm` SHA-256: `46e4e15367f1bc98c8718540d60980cdf3f7f1a6ed3c41609927a12e3ef964be`.
- The local server returned HTTP 200 for `index.html`; fetched JavaScript and
  Wasm hashes matched the bundle files and the existing
  `stage11-existing-bundle-keyboard-assets-2026-10-24.json` record.

This records the identity of the observed artifact. The existing build record
attributes it by build record and timestamps, not by an embedded source
manifest, so it does not prove that the artifact was built from the current
HEAD. No M1285 patch was applied, no query was typed, and VoiceOver was not
tested. This is baseline evidence only; post-patch accessible-name and
screen-reader acceptance remain open.
