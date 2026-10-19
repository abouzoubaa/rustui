# Upstream audit

Reviewed 2026-10-16. This records source/provenance checks, not successful integration
or GUI acceptance. The user approved the snapshot stack described below.

## Approved selection

- `gpui-pre = 0.3.9` (imported as `gpui`).
- `gpui-pre-platform = 0.3.9` (imported as `gpui_platform`).
- `gpui-component = 0.7.3`.

The published snapshot metadata records official Zed revision
`93d745668805029476e6d7fbb19652116fed51de`. Longbridge publishes this packaging;
it is distinct from Zed's published `gpui` release. Its publishing process renames
crates and adjusts packaging dependencies/features, so it must not be described as
an exact byte-for-byte copy of the original Zed workspace.

The user requested both dependency sources and compiled output inside RustUI.
Sources are vendored under `vendor/` with original licenses/checksums, and
compiled artifacts use `target/headstart/`. GPUI Fast remains disabled.

## Newer release review

| Candidate | Verified source | Finding |
| --- | --- | --- |
| GPUI Components 0.7.3 | crates.io manifest and release v0.7.3 (2026-10-12) | Renamed repository: longbridge/gpui-kit. Depends on package `gpui-pre = 0.3.9` under the name `gpui` |
| gpui-pre 0.3.9 | Published manifest and crates.io owner listing | Describes a snapshot of Zed `93d7456`; published by `huacnlee`; no packaged VCS info found |
| Official GPUI 0.2.2 | crates.io manifest and owner listing | Latest non-yanked stable published version; Zed repository and Zed publishing team |
| GPUI Components 0.5.1 | Published manifest/VCS info | Uses official `gpui 0.2.2` and `gpui-macros 0.2.2`; latest stable 0.5 release |
| GPUI Components 0.6.0 | Published manifest | Already uses separately published `gpui-pre` |
| Zed main | GitHub commit API | Observed `2d4c4831943153f63ad0aeddf49c8f5a3d008e75`; current GPUI package still calls itself 0.2.2, but APIs must not be inferred from version alone |
| Components main | GitHub commit API | Observed `f8cf0bd964f9ef2e5b7be79f3a8fc391371feb4a`; uses `gpui-pre = 0.3.9` |

The earlier recommendation of official published `gpui = "=0.2.2"` and
`gpui-component = "=0.5.1"` was superseded by the user's explicit snapshot choice.
The comparison remains here as provenance history. A future latest-Git stack still
requires its own compatibility review and approval.

The published GPUI 0.2.2 package's VCS record identifies
`69e2130295c2649963eb639fc70b4f2ee8ea1624` with `dirty: true`.
Therefore its immutable registry package/checksum, not a claim of exact clean
Git equivalence, is the reproducible source identity.
Components 0.5.1 identifies `0f0ab35233212f8f3277028995caf0c41e13ee6c`.
Components 0.7.3 identifies `67265a36e260ffe03c5aaebdcd8d654dbf892eab`.

Sources:

- https://crates.io/api/v1/crates/gpui
- https://crates.io/api/v1/crates/gpui/owners
- https://crates.io/api/v1/crates/gpui-component
- https://crates.io/api/v1/crates/gpui-pre/owners
- https://github.com/longbridge/gpui-kit/releases/tag/v0.7.3
- https://github.com/zed-industries/zed/commit/2d4c4831943153f63ad0aeddf49c8f5a3d008e75
- https://github.com/longbridge/gpui-kit/commit/f8cf0bd964f9ef2e5b7be79f3a8fc391371feb4a

Published manifests were read from the crates.io versioned download endpoints;
source declarations were inspected, but transitive dependency resolution and
build compatibility still require the approved integration.

## Historical local reference review

This section preserves provenance for the initial audit only. Allui and the old
Chorus facade are not active implementation sources for standalone RustUI. Current
component decisions start from RustUI's code, GPUI Components, and GPUI.

Local reference roots, inspected during Phase 0:

- `/Users/Abium/Developer/chorus/crates/foundation/rustui`
- `/Users/Abium/Developer/chorus/crates/third-party`
- `/Users/Abium/Developer/chorus/crates/third-party/gpui_component`

Those dirty/local reference trees are not dependencies of this workspace.
The old facade imports Allui and cannot be reused unchanged under the new
zero-Allui requirement. No upstream source was copied into this scaffold.

| Subsystem | Assessment | Treatment |
| --- | --- | --- |
| Official GPUI ownership/layout/notifications | Reusable | Integrate after pin approval |
| Existing GPUI retained views, scene replay, text caches, virtualization | Reusable | Measure before adding RGPE caches |
| GPUI Components widgets/theme registry/locales | Adaptable | Thin adapters; avoid redundant state |
| Old RustUI public facade | Requires redesign | Remove Allui ownership and public types |
| Existing playground coupling | Requires redesign | Build standalone RustUI Lab |
| Drag retention work in UI Studio | Reference only | Retained content/overlay separation; no separate current GPUI drag patch identified |
| RustUI runtime themes and localization contracts | Incomplete | Approve native contracts before implementation |
| Advanced RGPE GPU work | Missing here | Planned, separately authorized |

The GPUI Fast reference review used
`longbridge/gpui-fast@fa133a1dd970532dce4ffaee8577438971e5dec9`.
Its [pinned UPSTREAM record](https://github.com/longbridge/gpui-fast/blob/fa133a1dd970532dce4ffaee8577438971e5dec9/UPSTREAM#L5-L10)
declares Zed revision `93d745668805029476e6d7fbb19652116fed51de`, the
same declared Zed base as the approved Longbridge snapshot. The fork and snapshot
packaging still have different source bytes and integration; this shared base
does not establish patch compatibility or byte identity.
It is not a RustUI dependency. The reviewed path disables retained-subtree and
scroll-layer reuse during active drag; selective drag retention therefore remains
an investigation, not an adopted implementation.

## Verified integration and remaining acceptance

Approved source identities are resolved in the genuine Cargo lockfile, and all
797 external packages are vendored locally with matching package checksums.
Allui is absent from the lockfile. Cargo's active macOS dependency graph has no
GPUI Fast packages enabled.

The Headstart native build succeeded with Cargo network access disabled. A final
unchanged build reused all 488 artifact records, including GPUI, its platform
backend, and Components. Every artifact path is under `target/headstart/`.
See `docs/BUILDING.md` for commands, evidence paths, warnings, and limitations.

A native Lab window, widget functionality, accessibility, renderer correctness,
and performance acceptance remain separate implementation/validation work.
