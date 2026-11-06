# M1222 — Stage 8 Tree `aria-level` acceptance run sheet

## Purpose and gate

Use this sheet to verify the proposed one-based hierarchy level on each outer
TreeItem row. The change belongs in the pinned `gpui-base` Tree renderer, where
the row and its `TreeEntry::depth()` are available. RustUI's `ListItem` remains
a child of that row and must not create a second TreeItem.

This sheet records acceptance steps; it does not approve patch application.
Before changing vendored source, record the pinned upstream owner's decision
and the separate approval for the versioned local patch, as required by the
M1222 proposal. Do not change a dependency revision or fetch a missing test
dependency as part of this run.

## Current gate — 2026-10-25

The renderer gap is confirmed: the pinned `gpui-base` Tree owns the outer
`TreeItem` row and has its zero-based `TreeEntry::depth()`, while the pinned
GPUI element API already exposes `.aria_level(usize)`. RustUI's inner
`ListItem` cannot set the outer row's level. The current worktree contains no
record of the pinned upstream owner's decision or approval to apply a local
patch, and `patches/README.md` still says no patches are installed. The active
vendored renderer is unchanged.

The referenced level-patch file and patch-checker script are also absent from
this worktree. If upstream cannot take the change and local patch application
is approved, prepare those versioned artifacts with exact source identities
before beginning the preflight below.

**Smallest next action:** obtain and record the pinned upstream source owner's
decision on adding `.aria_level(entry.depth() + 1)` to the outer row. If the
owner cannot take it, obtain separate approval for the exact versioned local
patch before patch validation or application. After approval, follow the
preflight and acceptance steps below. Until then, no renderer source change is
authorized by this run sheet. The roadmap's broader Tree keyboard, screen
reader, and platform acceptance also remains open; these are later evidence
gates, not substitutes for the source-owner decision.

## Source and patch identity

Verify these values against the approved patch package and the actual
acceptance worktree before applying or testing anything:

| Item | Expected identity |
| --- | --- |
| Package under test | `gpui-base 0.7.3` from Longbridge `gpui-kit` revision `67265a36e260ffe03c5aaebdcd8d654dbf892eab` |
| GPUI element API | `gpui-pre 0.3.9`, pinned Zed snapshot revision `93d745668805029476e6d7fbb19652116fed51de` |
| GPUI Components | `gpui-component 0.7.3` |
| Registry archive checksum | `2e9955b6f4b048fd50ca15fc8e225d80bb00ba9ebc8fecdddde5ad26dbef9270` |
| Pristine `src/tree.rs` checksum in `.cargo-checksum.json` | `e74e4fcdef6ccec174e3e6ced50f108433e726f3e3d46f3539b1f41d5944e5df` |
| Proposed patch | `patches/proposed/gpui-tree-aria-level-v1.patch`, SHA-256 `67324d2ff67e53c6bc62b7a41609957bca559f0f70e0964fcec17de2ca48f57b` |
| Expected patch input Tree hash | `9ef2efa0a44af810eef3088343b0838179d5e952c6ca8734e5306d5c74736924` |
| Expected patched Tree hash | `4dae8754abef34b1173e0b69b4dfb80483ae61545e2dd3614fc259dfd38fe6fb` |

The patch input is the recorded postimage of the approved Tree `aria-disabled`
patch. A pristine checkout has a different Tree hash. If the source does not
match the exact expected input, stop and reconcile the approved patch sequence;
do not apply with fuzz, edit the vendored file by hand, or overwrite the source.

## Preflight and focused regression

- [ ] Use the approved, isolated worktree and record its branch and base commit.
- [ ] Record the upstream-owner decision and patch-application approval reference.
- [ ] Confirm the package, registry checksum, pristine source metadata, patch
  hash, and exact Tree input hash above. Record all actual values.
- [ ] Run the package's read-only patch checker before application:

  ```sh
  python3 scripts/apply_gpui_tree_aria_level_patch.py --check
  ```

  It must verify the exact patch/source identity and apply/reverse hash
  round-trip in temporary copies. Record the output and exit status.
- [ ] Apply only through the approved versioned patch process. Confirm the
  resulting Tree source hash equals the expected patched Tree hash and inspect
  the scoped diff: `.aria_level(entry.depth() + 1)` must be on the outer row
  beside its existing TreeItem role and accessible label.
- [ ] Run the focused regression when its dependencies are available:

  ```sh
  ./scripts/cargo-headstart test \
    --manifest-path vendor/gpui-base-0.7.3/Cargo.toml \
    --lib tree::tests::tree_item_accessibility_levels_follow_entry_depth \
    --locked --offline
  ```

  The proposal records that this locked/offline test is currently unavailable
  because `futures-concurrency` is missing from the pinned offline vendor
  sources. If it is still unavailable, record the exact diagnostic as
  `not_run` and leave the regression gate open. Do not fetch or add a dependency
  without separate approval.

The proposed unit regression checks the nested chain at levels 1, 2, 3, and 4
through GPUI's accessibility-tree JSON. The native story below supplies the
additional same-level sibling and multiple-root cases.

## Expected visible rows

In the expanded Playground reconciliation Tree, each row must appear exactly
once as an outer `TreeItem`, with the following accessible name and one-based
level:

| Accessible name | Expected level |
| --- | ---: |
| Workspace | 1 |
| Branch | 2 |
| Nested child folder | 3 |
| Grandchild leaf | 4 |
| Stable sibling | 2 |
| Archive | 1 |
| Archived item | 2 |

The level follows the current visible hierarchy: roots are 1, and each child
adds 1. Collapsing a folder removes its hidden descendants from the visible
accessibility tree; it does not renumber the remaining rows. Reordering roots,
relabeling, or changing selection must not change levels for rows whose
hierarchy is unchanged.

## Native Playground acceptance

Use the canonical RustUI Playground `Framework Demos > Tree` reconciliation
story (`tree-reconciliation-tree`) on macOS. Reuse the established disposable
validation application and bundle ID `dev.rustui.playground.validation`.
Record the binary/source identity used for the run; an older M476 observation
is background evidence only and does not validate this patch.

- [ ] Capture the native accessibility hierarchy with all branches expanded.
      Confirm the seven name/level pairs above, exactly one outer TreeItem per
      row, and the applicable selected and expanded state.
- [ ] Confirm the child ListItem does not introduce a nested duplicate
      TreeItem. Record whether the row name, level, and state are attached to
      the outer item.
- [ ] Reverse root order. Confirm Workspace and Archive remain at level 1 and
      each child keeps its expected level and identity.
- [ ] Relabel rows. Confirm accessible names update while levels stay the same.
- [ ] Collapse and restore Workspace. Confirm descendants disappear and return
      with levels 2, 3, and 4; Workspace remains level 1.
- [ ] Select Workspace, then Grandchild. Confirm the selected state follows
      the row and existing owner-selection behavior remains intact.
- [ ] Activate Grandchild and confirm one leaf activation. Remove Branch and
      restore it; confirm selection normalization and the remaining sibling
      levels are correct.
- [ ] Exercise Up/Down selection and Left/Right folder expansion. Record the
      focused accessibility element; do not infer child focus from root-window
      focus alone.
- [ ] With VoiceOver enabled, navigate the rows. Record the spoken row name,
      hierarchy level, selection, and expanded/collapsed state where applicable.
      Mark VoiceOver `not_run` if unavailable; visible text is not a substitute.

The prior M476 record established bounded macOS 27 visibility, labels,
selection, expansion, leaf activation, and branch removal/restoration. It
reported child focus ownership unverified and VoiceOver `not_run`; it did not
measure `aria-level` and must not be reused as patch acceptance.

## Browser evidence boundary

Run a browser check only if the canonical Playground Web build actually renders
this Tree story. First record the exact bundle/build identity, route, and
whether the story is reachable. Inspect the browser accessibility tree and
verify the same seven names and levels, one TreeItem per row, plus the relevant
state and interactions.

If the Tree is not rendered in Web, mark browser acceptance `not_applicable`
with the reason and evidence. Do not infer DOM or browser accessibility support
from native observations, source presence, a canvas screenshot, or another
story. A browser screen-reader check is a separate result from inspecting the
browser accessibility tree.

## Evidence record

Complete this record for each acceptance attempt. Preserve failed and partial
observations; do not replace them with a summary claim.

```text
Date/time and timezone:
Milestone / run identifier:
Worktree, branch, base commit:
Pinned package versions and repository revision:
Registry checksum / pristine checksum metadata:
Patch path and SHA-256:
Tree input hash / patched hash:
Upstream-owner decision and patch-approval reference:
Patch checker command, output, exit status:
Focused regression command, exact output, exit status:
Regression status: passed / failed / not_run
GUI app build command and binary SHA-256:
Bundle ID and validation bundle path:
Platform and OS version:
Story route and stable tree ID:
Native hierarchy artifact path and observations:
Keyboard interactions and focus observation:
VoiceOver status and recorded announcements:
Browser bundle/build and route, if applicable:
Browser accessibility artifact and observations, if applicable:
Cleanup: app quit / bundle removed / no process remains:
Result: passed / partial / failed / not_run
Open limitations and follow-up:
```

## Acceptance decision

Record `passed` only when the exact source and patch identities match, the
focused regression passes, and native hierarchy/interaction and VoiceOver
observations establish the requested behavior. Record browser status separately
and only when the Tree is rendered there. Missing dependencies, inaccessible
assistive technology, unavailable routes, or incomplete observations remain
explicit open gates; they are not passes. This run sheet itself does not apply
the patch or close the broader Stage 8 acceptance goal.
