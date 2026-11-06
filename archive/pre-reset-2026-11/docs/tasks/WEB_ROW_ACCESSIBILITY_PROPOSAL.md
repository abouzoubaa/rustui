# Web row accessibility: exact same-version proposal

Status: prepared and validated in isolated copies; **not promoted**. Current
selected vendor sources remain unchanged. The source manifest is
[`gpui-row-accessibility-series-v1.json`](../../patches/proposed/gpui-row-accessibility-series-v1.json).

User direction on 2026-10-26 defers accessibility work alongside VoiceOver.
Promotion and runtime accessibility acceptance are **undone and deferred**.
Keep this prepared proposal unapplied while functional release work proceeds.

## Observed problem and prepared repair

| Source | Current behavior | Proposed behavior |
|---|---|---|
| GPUI-pre 0.3.9 `src/elements/div.rs` | Primary-mousedown-only rows have working pointer behavior, but advertise no AccessKit Click. The Web adapter correctly rejects unadvertised actions, so semantic Tree-row activation does nothing. | Advertise Click for an explicit primary-button mousedown listener. Reuse GPUI's existing Click fallback, which dispatches a normal primary down/up pair. Secondary-only and outside listeners stay excluded. |
| Components 0.7.3 `src/list/list.rs` | Selectable Lists project list/listitem with `aria-selected`, which is not a selected-item role. The catalog's named inner label is flattened separately from its selectable parent. | Selectable Lists project listbox/option; ordinary nonselectable Lists retain list/listitem and omit selected state. Existing GPUI selection, confirmation, focus and owner scheduling remain authoritative. |

Both patches retain the approved package versions and upstream revisions.
Source pre/postimages, patch hashes, Cargo/provenance checksums and compatibility
limits are recorded in the manifest. No dependency versions were added or
changed by isolated validation; only the local probe package is new.

## Executed validation

- Exact disposable apply/reverse and unchanged selected-source checks pass.
- **3/3 checker regressions pass**, including corrupt patch and partial-source
  rejection without source repair.
- The isolated Headstart probe passes **4/4 tests**: primary mousedown Click,
  secondary/outside exclusions, existing explicit Click, and actual rendered
  selectable/nonselectable List roles and selected-state validity. Its doctest
  target contains zero cases.
- The broader source checkpoint passes **584 Python tests**, **27 JavaScript
  semantic/client fixtures**, packaging fixtures and source guards. These
  checks do not compile the selected Rust workspace or establish GUI acceptance.
- Portable probe source and manifest template are retained at
  `patches/proposed/row-accessibility-probe.rs` and
  `patches/proposed/row-accessibility-probe.Cargo.toml.in`. The template replaces
  only absolute checkout/candidate roots; it preserves the selected package
  overrides and approved profile settings.
- Raw logs, resolved metadata, exact probe lockfile and host identity are under
  `target/headstart/evidence/row-accessibility-proposal-20261026/`; the compact
  result is `validation.json`. Initial draft field-visibility and probe-trait
  errors were corrected before the passing run; their logs remain separate.

Recheck source/patch integrity with:

```sh
python3 scripts/check_gpui_row_accessibility_proposal.py
python3 -m unittest scripts.tests.test_gpui_row_accessibility_proposal
```

To reproduce the compiled probe, copy both selected vendor packages into a new
disposable `@CANDIDATE_ROOT@/vendor/` tree, apply the two manifest patches there,
substitute that root and `@RUSTUI_ROOT@` into the Cargo template, and copy the
probe Rust source to its `src/lib.rs`. Seed its lockfile from the repository
lock, resolve offline, and reject any additional upstream package/version pair.
Run `./scripts/cargo-headstart test --manifest-path <probe>/Cargo.toml
--profile headstart --locked --offline` from the registered checkout, using a
persistent target directory under `target/headstart/`.

## Promotion decision

Recommendation: promote both exact patches through the reproducible patch
process, then rebuild and validate actual native/Web row activation, keyboard
selection, disabled behavior and current-owner updates. This proposal does not
establish that runtime acceptance or screen-reader speech.

Keeping the current stack preserves existing pointer/keyboard behavior but
leaves semantic row acceptance open. A RustUI-specific second event dispatcher
would duplicate GPUI ownership and is not recommended. Remove the patches once
the matching official upstream behavior is selected and verified.

The current repository instructions require approval before promoting a new
upstream patch. This proposal is additional to the already selected Web v9 and
Tree accessibility series; no promotion is inferred from their earlier approval.
