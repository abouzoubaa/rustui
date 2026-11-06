# Tree disabled keyboard behavior proposal

Status: the RustUI wrapper guard is implemented in M389; the upstream patch
below remains proposed only. It is not applied to `vendor/`, and no upstream
revision or package source has been promoted.

## Contract

Disabled Tree folders may remain reachable and selectable with keyboard
navigation. At the RustUI `TreeView` boundary, `Confirm`, `SelectLeft`, and
`SelectRight` are stopped while a disabled folder is selected, preserving its
expanded state and preventing expansion events. The pinned GPUI Tree row also
suppresses pointer handling for disabled entries.

M389 implements this wrapper guard, with a focused Tree unit suite. The
proposed upstream guard remains a separate option for enforcing the
disabled-folder invariant inside `TreeState::toggle_expand`, including callers
that bypass RustUI's wrapper.

## Proposed change

`patches/proposed/gpui-tree-disabled-keyboard-v1.patch` adds the disabled check
to the shared `TreeState::toggle_expand` path. This is broader than M389's
RustUI wrapper guard and would also protect direct `TreeState` callers. The
patch adds the focused `disabled_folders_ignore_expansion_requests` GPUI
regression for both initially collapsed and expanded folders, checking visible
entries and expansion events.

This proposal builds on M269's Tree `aria-disabled` patch. Its preimage is the
M269 postimage for `vendor/gpui-base-0.7.3/src/tree.rs`:

- M269 patch: `patches/approved/gpui-tree-aria-disabled-v1.patch`, SHA-256
  `5762dfec8eba4b723bc1b851c527c75f18e78d76d050fd8bab4485a63ceb14d1`
- Required Tree preimage SHA-256:
  `9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924`
- Proposed Tree postimage SHA-256:
  `4b1c4d38493f0c26b9b3c08d1baa89372de33b013320426ae8f07ede3be89db6`
- Proposed patch SHA-256:
  `6734386f8db13f51f5330a05e3f5c505f59f22218caeff869a800f6fea7dc47b`

The validator `scripts/check_gpui_tree_disabled_keyboard_patch.py` reads the
expected source, copies only that file into a temporary directory, applies the
proposed patch there with zero fuzz, verifies the patch and resulting file
hashes, and runs `rustfmt --check` on the isolated postimage. It never writes to
the active vendored source.

## Validation and limits

Focused isolated application check:

```sh
python3 scripts/check_gpui_tree_disabled_keyboard_patch.py
```

The isolated apply/hash and formatting checks passed on 2026-10-20. The
proposed GPUI regression was not compiled or run; this record verifies exact
patch application and the presence of that regression in its postimage. M269's
existing GPUI test-target prerequisite failure is recorded in
`target/headstart/evidence/m269-gpui-pre-a11y-test-20261020.log`. Native
keyboard, pointer, and accessibility acceptance remain open. Promotion requires
the separate approved patch process; this proposal does not authorize changing
vendored files.

M389's focused RustUI Tree suite passed 15/15 in
`target/headstart/evidence/tree-disabled-folder-keyboard-m389-test-20261021.log`.
Native keyboard and accessibility acceptance remain open.

## Focused regression attempt — 2026-10-20

The exact-hash patch was applied only to a disposable copy under
`target/headstart/candidates/m275-disabled-tree-regression-20261020/`; active
vendor source was not changed. The locked/offline focused test command stopped
during dependency resolution because `gpui-fast` is absent from the configured
vendor directory. No Rust test code compiled or ran. Log:
`target/headstart/candidates/m275-disabled-tree-regression-20261020/m275-focused-test.log`.
