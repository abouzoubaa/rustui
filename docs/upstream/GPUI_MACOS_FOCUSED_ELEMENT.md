# macOS focused accessibility element

The approved same-version patch installs AccessKit's existing focus forwarder
on GPUIWindow and GPUIPanel immediately after those Objective-C classes are
registered. GPUI's content view already owns the AccessKit adapter and the
framework's real input focus. The two calls let a window's
`accessibilityFocusedUIElement` query reach that view instead of stopping at the
window itself. No GPUI/Components revision, runtime or rendering strategy changes.

Cargo selects the same pinned version through its local vendor path. This is
required: Cargo treats registry packages as immutable and can reuse an original
cached artifact even after a verified vendor patch. The verifier rejects a
registry-selected backend and checks the root path patch and lockfile identity.
The original upstream checksum remains recorded in the vendor provenance.

With that selection in place, install and verify through the exact-image helper:

```sh
python3 scripts/apply_gpui_macos_focused_element_patch.py --check
python3 scripts/apply_gpui_macos_focused_element_patch.py --apply
python3 scripts/apply_gpui_macos_focused_element_patch.py --verify
```

An installed checkout needs only `--verify`; `--reverse` restores exact upstream
bytes. The helper checks the manifest and patch hashes, both package lock rows,
vendored upstream checksums, source revisions, the existing AccessKit API and
license texts. It requires no global Cargo cache. Unknown source is rejected and
failed replacement rolls back atomically. Six fixture tests cover lifecycle,
unknown source, registry selection, license/API tampering and rollback, including installed POST
source and an empty CARGO_HOME.

Before this patch, native AX text entry worked but the focused element remained
the window. The selected local backend compiled in the optimized native build;
the owned-bundle retest confirms the focused named search field, clearing with
Command+A/Backspace, keyboard actions, and forward/reverse Tab skipping the
disabled button. Five of eight bounded native checks pass. Coordinate input
still reports `noWindowsAvailable`; a joint capture shows AX Count:2 with an
image still showing Count:0. Their service/capture-versus-product cause remains
unconfirmed. VoiceOver was not run. Native visual, physical-input and full
accessibility acceptance remain open.

Evidence, executable hash and cleanup record:
`/tmp/rustui-preservation-20261025/native-focused-element-v1/observations.json`.
Only the verified owned process was stopped. Its bundle and standalone
executable were removed; the GUI slot was released and build caches retained.
