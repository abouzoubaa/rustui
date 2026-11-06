# Selected Web semantic and native-input adapter (v9)

This approved, reproducible series keeps GPUI-pre 0.3.9 and Components/Base
0.7.3 pinned. The versioned patch and manifest select one composed source image;
legacy accessible-name, Enter and keyboard-focus helpers delegate to it.
Vendored source is installed by the exact-image helper, never edited directly.

The adapter exports GPUI roles, names, state, selection and supported actions to
browser semantic elements. It routes actions back through GPUI, preserves the
native input owner during compatible snapshots and normalizes physical bounds
to CSS pixels. Protected fields use a password-type owner with semantic value
suppression. It does not add another rendering or reactive runtime.

Editing uses a verified UTF-16 context window and reconciles both text and
selection before forwarding edits. Base's generic input handler implements the
native selection setter used by Input, Textarea and Editor. The bridge checks
that the model accepted that selection before adopting it; default no-op
handlers fail closed. Command shortcuts are enabled on Wasm alongside Control
shortcuts. GPUI still owns key dispatch, bindings and canonical text state.

## Reproduction

On the exact unmodified image described by the manifest, install the approved
name/Enter prerequisites, then the selected adapter:

```sh
python3 scripts/apply_gpui_web_semantic_adapter.py --apply-prerequisites
python3 scripts/apply_gpui_web_semantic_adapter.py --check
python3 scripts/apply_gpui_web_semantic_adapter.py --apply
python3 scripts/apply_gpui_web_semantic_adapter.py --verify
```

The selected checkout also applies the approved RTL wrapped-row patch to a
shared Web source file. Verify that installed stack with
`python3 scripts/verify_gpui_web_patch_stack.py`; it reverses the RTL layer
only in a temporary image before proving the semantic layer. The `--verify`
command above applies to the intermediate semantic-only image. `--reverse`
returns that intermediate image to the exact prerequisite image;
`--reverse-prerequisites` returns from there to the original image. The helper
checks preimages, patch digests, postimages,
upstream checksums/licenses and the composed keyboard baseline. It stages
multi-file replacement and rolls back failed writes. Unknown source images
are rejected. Keep the separate Tree and disabled-state patch selections.

## Current evidence and limits

The macOS workspace all-target check and optimized Web release build passed.
The release driver ran 18 actual Rust edit/selection planner tests. The selected
stack passed 7 lifecycle checks, 9 source checks and 19 JavaScript fixtures; the
complete Python checkpoint passed 575 tests. The two added upstream Base GPUI
regressions are prepared inside its existing test module but were not run as
standalone upstream tests.

Current browser observations cover named input ownership, settled/repeated/
empty replacement, Command+A deletion, Unicode selection/replacement, keyboard
catalog navigation and semantic row activation. Playground's secure-field story
uses independent state so its public labels and inputs do not echo that value.
Protected-value inspection is inconclusive; selection movement and clearing
were observed, but exact protected replacement is not claimed.

Real OS IME composition, clipboard roundtrips, screen readers, broader browser
coverage and final performance acceptance remain open. Native AX actions are
separate evidence: coordinate input still reports `noWindowsAvailable` and rendered captures
disagree with current AX state. The separately selected native focus-forwarder
patch passes its named-field AX retest; physical/visual/VoiceOver gates stay open. Do not infer native visual or VoiceOver acceptance from these tests.

Historical development artifacts and failed-build evidence are preserved under
`/tmp/rustui-preservation-20261025` and the frozen `/tmp/rustui-web-semantic-adapter-*`
packages. Current browser provenance and observations are in
`/tmp/rustui-preservation-20261025/browser/semantic-v9`.
