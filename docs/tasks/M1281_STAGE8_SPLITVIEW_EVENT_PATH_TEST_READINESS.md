# M1281 — Stage 8 / SplitView divider event-path test readiness

**Status: blocked; no dependency or source change made.**

The requested regression must dispatch a physical arrow key through the
rendered divider's `on_key_down` handler and observe the resulting native
`ResizableState` change. The existing
`divider_keyboard_adjustment_moves_physically_and_clamps` test exercises the
pure key-to-size helper; it does not dispatch a GPUI event or verify the
production `ResizableState::resize_panel` call.

## Test-support availability

The pinned GPUI and GPUI Components sources do expose `test-support` features
and event-dispatch helpers. However, the locked, offline feature tree resolved
for `rustui` contains no `test-support` feature edge, and the RustUI crate has
no existing `TestAppContext`/`VisualTestContext` usage or event-dispatch test
harness:

```sh
./scripts/cargo-headstart tree -p rustui -e features -i gpui-pre --locked --offline
```

The command exited successfully; filtering its output for `test-support`
returned zero lines. The check was repeated during this readiness pass and
again returned zero lines. Pinned GPUI, GPUI Base, GPUI Components, and GPUI
platform feature declarations fan test support into additional GPUI and
platform/support crates. RustUI does not currently declare or activate that
test harness. Activating it would require RustUI Cargo feature wiring and a
working build of the resulting feature closure; neither is available in the
current checkout. No feature, dependency revision, or vendored source was
changed to enable it.

## Existing source-level regression attempt

Attempted the nearest supported focused regression:

```sh
./scripts/cargo-headstart test -p rustui --lib divider_keyboard_adjustment_moves_physically_and_clamps --locked --offline
```

It exited 101 before running the test. Rustc reported `E0463: can't find crate`
for `gpui`, `gpui_component`, `rustui_core`, `rustui_i18n`, `chrono`,
`gpui_base`, and `serde`. This run provides no passing regression evidence.

## Scope and next step

No M1281 production event-path test is supported by the current RustUI feature
graph, and the existing helper test is not equivalent coverage. Resolve the
Headstart dependency-artifact failure and review the complete pinned
test-support feature closure before adding or enabling a RustUI event-dispatch
harness. Do not change upstream revisions as part of that work. M1281 remains
blocked at readiness: the production event path has not been exercised by a
RustUI test, and this pass made no source or dependency changes.

The current remaining-audit record lists M1280 as implemented with a focused
source regression; M976, M1142, M1190, and M1066 are also recorded complete.
M1222 remains separately approval-gated. No additional un-gated Stage 8 source
defect was established in this pass.
