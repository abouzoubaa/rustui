# macOS 27 HIG static validation and scope deviation — 2026-10-22

## Scope

This report records static file and JSON checks for the design-only artifacts in `hig/`. It does not claim runtime, build, test, accessibility, or native macOS acceptance. No files outside `hig/` were intentionally edited or restored for this validation.

## Static validation

| Check | Result | Method and limit |
|---|---|---|
| Editable theme vs production snapshot | Match | `hig/macos27.json` and `hig/macos27.production-baseline.2026-10-22.json` are both 820,587 bytes with SHA-256 `5a4a81adb6af226d59af916a4feadf575b23447b7cee2517882e9bdf2453e01e`. |
| Preserved pre-design snapshot | Recorded | `hig/macos27.hig-pre-design-baseline.2026-10-22.json` is 811,905 bytes with SHA-256 `903c483647ee2c9cf371d98e8c1f051c6ff58ee8ff21b1903e5bd15fa1ee5e3e`. |
| Theme/catalogue counts | Match documented counts | `macos27.json` and `full.macos27.json` each parse as 41 component IDs and 419 style rules. |
| Design-only candidate inventory | Preserved | 31 source-present contracts, 5 absent reusable patterns, 11 native integration candidates, and 9 composition/reference examples. The candidate declares `runtime_compatible: false`. |
| Visual reference map | Verified | All 86 CSV paths exist and match their listed file sizes and SHA-256 values; 46 are under `rustui-images/` and 40 under `ui-images/`. |

### Production-only path count reconciliation

The earlier value of **96 production-only paths** is reproducible when “path” means the first absent JSON object key on each branch: recursively compare objects, record an absent key, and stop descending under that missing parent. This yields 96 production-only structural paths, zero HIG-only structural paths, and zero differing values at shared paths between the production snapshot and the pre-design HIG snapshot.

The 96 structural additions group as follows: `source` 3, `token_sources` 12, `theme_roles` 13, `components` 66, and `derivation` 2. All other root groups contribute zero.

A leaf-flattening comparison uses a different unit. It expands values beneath each absent parent and reports 242 production-only leaf paths. Those are descendants of the same 96 missing structural paths, not 242 additional independent object-key additions. Empty-object placeholders in the older snapshot are not counted as HIG-only data paths. The earlier reconciliation used the structural-path convention; the count is not being reduced to conceal the leaf-level difference.

After reconciliation, the editable `hig/macos27.json` equals the production snapshot exactly. This static comparison does not establish that a renderer consumes every serialized property.

## Scope deviation: Trail cache refresh outside `hig/`

A required Trail query was run before the boundary correction. Its output said: `[trail] refreshed the graph (2 files changed) before answering`. That output did not name the two files. The query ran from the repository root and refreshed ignored files under `.trail/cache/`, outside the authorized HIG write boundary.

The following files are currently observable with recent modification times. Their current size, modification time, and SHA-256 are recorded as post-observation evidence only:

| Path | Observed mtime (UTC) | Bytes | Current SHA-256 |
|---|---:|---:|---|
| `.trail/cache/.cache/ask-index.json` | 2026-10-22 00:53:12 | 2,724,369 | `d74d60252078139487dc0a86eb95cc89c26bc38786f141a0e2003b08cc2d2699` |
| `.trail/cache/.cache/extract.4681a877fedd64d4.json` | 2026-10-22 00:53:12 | 11,788,460 | `a3b376cc9b16f1210de60ad69909ee9ae199eb63bbc9068e2af65fddc419364c` |
| `.trail/cache/.cache/fingerprint.4681a877fedd64d4.json` | 2026-10-22 00:53:12 | 41,482 | `358a9ba3f594983610e7b705c1eb34526109753266b527a2b93a32ec47277899` |
| `.trail/cache/.graph/wiring.json` | 2026-10-22 00:53:12 | 4,407,920 | `5246cab6e019c35c5895ce075a72927028fb19e5337a33f7245d61ea6d3afdba` |
| `.trail/cache/.cache/trail-branches.json` | 2026-10-22 00:56:30 | 8,968 | `542e0bac2700bc4e7143db8b597820458f9fa9b329ef54f45e7d4e9e7671782c` |

The paths above are the exact recently modified cache files visible from filesystem metadata; without pre-query hashes, a filesystem journal, or Trail's per-file write log, it cannot be proven that these are the only files changed by the query or mapped to its “2 files changed” message. No pre-query contents or hashes were saved, and Git does not track these ignored cache files, so the original contents cannot be recovered or proven from the available evidence. These files have not been deleted, restored, or otherwise altered after discovery.

## Validation boundary

Static JSON parsing, counts, hashes, and path comparisons were checked. No build, test, runtime coverage report, GUI inspection, VoiceOver check, native AppKit integration, or macOS 27 SDK validation was run as part of this report. A passing static check must not be presented as runtime or platform acceptance.

## Owner reference coverage update — 2026-10-22

Ran `python3 hig/validate_owner_component_coverage.py` after adding the complete owner-reference matrix and candidate metadata. The script reads only the owner source, HIG JSON/baseline, candidate, and matrix under `hig/`, plus the runtime registry source for row annotation was inspected separately; the validation script itself writes nothing.

| Check | Result |
|---|---|
| Exact named owner rows | **148 / 148**, in source order, across all **16 / 16** named sections; section labels and exact bullet text compared. |
| Supplemental owner items | **28 / 28** exact rows: 8 appearance requirements, 11 version-claim families, and 9 supplied links. |
| Required row fields | Populated for every row: canonical ID, ownership, Apple claim state, HIG JSON, code/runtime, design candidate, proposed contract, accessibility/input/localization, fallback, and validation plan. |
| HIG editable JSON vs preserved production snapshot | Match, SHA-256 `5a4a81adb6af226d59af916a4feadf575b23447b7cee2517882e9bdf2453e01e`. |
| Current production theme file vs preserved snapshot | Match at the same SHA-256; checked read-only during this update. |
| Preserved baseline inventory | **41 IDs / 419 styles**. No `hig/macos27.json` value was changed by this reconciliation. |
| Design candidate | Parses; points to both coverage ledgers; `runtime_compatible` remains `false`. Existing 31/5/11/9 candidate counts are preserved; cross-cutting requirements and behavior gaps are separate non-primitive metadata. |
| Visual references | All **86** stored image paths exist under `hig/` and match the mapped size and SHA-256. |
| Static script result | `PASS: static HIG-local owner coverage checks`. |

The matrix category totals are 17 system-owned, 5 native-system integrations, 51 native-host/composition, 64 RustUI/style/composition, 10 native material/API, and 1 retired/unsupported. These classify rows; they are not feature-support totals. No build, test, GUI, accessibility-tree, VoiceOver, or target-macOS acceptance was run.

## Visual fidelity acceptance gate

The acceptance criterion for subsequent design work is recorded in [`MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md`](MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md) and linked from the candidate. Current status is **NOT DEMONSTRATED**: no macOS 27 runtime/reference captures, measured native values, interaction observations, or VoiceOver outcomes are present. macOS 26.1 SDK evidence is not a macOS 27 substitute. Generated images remain supplementary; coverage counts and this static validation do not establish fidelity or compliance.
