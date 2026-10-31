User approval recorded 2026-10-25: "I approve everything just proceed." The prepared exact same-version series is authorized for reproducible promotion and focused validation; native/browser acceptance remains open.

# Full Tree root/name/level promotion contract

This is the prepared post-approval integration contract. It does not itself prove
human promotion approval, source installation, native AX or VoiceOver acceptance.
The coherent promotion operation is the integration package followed by the exact
approved series helper's PRE check, apply and read-only POST verify.

## Scope and source authority

The source baseline already contains approved Tree aria-disabled v1. The complete
new series uses exactly these same-version artifacts, in order:

1. `patches/approved/gpui-pre-test-a11y-activation-v1.patch` — test-only GPUI
   TestWindow accessibility activation (`gpui-pre 0.3.9`).
2. `patches/approved/gpui-tree-aria-level-v1.patch` — one-based levels on existing
   outer TreeItem rows, including the corrected renderer regression which reads
   the debug accessibility `nodes` object map (`gpui-base 0.7.3`). Its exact patch
   SHA-256 is `7729270cd97bfc74039e05d5142d522b78cc9bd563a825ccaa33caeb2d8f77f7`.
3. `patches/approved/gpui-component-tree-accessible-name-v1.patch` — accessible-name
   builders on the existing base/component Tree root (`gpui-base 0.7.3` and
   `gpui-component 0.7.3`). No additional semantic root or rendering runtime.

The RustUI integration supplies the root's caller-provided/localized accessible
name and retains its source/rendered regression and English/French/Arabic locale
records. All exact patch/source/provenance hashes and ordered paths are in
`patches/approved/gpui-tree-accessibility-series-v1.json`. The original review
artifacts remain under `proposed/`. No version, upstream revision, checksum
metadata, license, Cargo selection or GPUI Fast change is included.

TestWindow activation is required because its former default accessibility init
was a no-op: a rendered debug accessibility test could otherwise produce no tree.
That backend test support is not evidence about the OS accessibility tree or
VoiceOver. The corrected full scope must not be replaced with an old level-only
patch or tests which inspect source strings instead of the rendered node map.

## Exact replay and compatibility

`scripts/apply_gpui_tree_accessibility_series.py` uses only the approved artifact
paths. `--check` accepts exact PRE and checks forward/reverse in disposable copies;
`--apply` installs all three exact POST images; `--verify` accepts exact POST and
is read-only; `--reverse` restores all three exact PRE images, leaving the older
aria-disabled baseline installed. Unknown/partial images or changed artifacts
fail before source installation. The installer retains transaction rollback.

POST verification reverses the new series in copies, proves the older
aria-disabled reverse/replay against pristine baseline source hashes, and replays
the complete new series. The old aria-disabled `--check` accepts a composed Tree
only by delegating to this full proof; it cannot silently accept one changed Tree
hash while the component/TestWindow images are missing. Its mutation modes require
the new series to be reversed first. Standalone aria-level mutation is retired;
its compatibility entry point only performs read-only full-series checks.

CI and the source/macOS/Web release entry points require exact POST and composed
baseline checks before further gates. The source fixture suite covers PRE check,
apply, POST verify, composed baseline check, reverse, wrong-mode/unknown/partial
rejection, changed artifacts, legacy read-only compatibility and partial-install
rollback for all three sources. Package checksum/license provenance is preserved.

## Evidence and remaining acceptance

The preparation session passed the seven disposable lifecycle fixtures from a
POST validation root, direct PRE/apply/POST-verify/baseline-check/reverse commands,
and release shell syntax. Logs are retained with the external prepared package.
No Cargo/build/app/browser/native AX/VoiceOver run was performed for this package.

After explicit promotion approval and exact installation, run the focused GPUI
renderer and RustUI Tree regressions, relevant approved Headstart checks, and
source-bound native/browser accessibility acceptance separately. Verify the
existing named Tree root, stable selected/expanded/disabled TreeItem rows,
one-based levels, activation behavior, locale updates and no duplicate semantic
nodes. Actual VoiceOver, browser, localization/RTL and cross-platform acceptance
remain open until observed and recorded.

Rollback is the exact series reverse plus a scoped reversal of this integration
package, preserving unrelated work. Remove the patches only when a separately
approved upstream snapshot provides equivalent root naming, row levels and test
backend behavior and its acceptance evidence has been verified.
