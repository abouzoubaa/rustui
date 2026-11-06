# P2C — Pinned GPUI timing hooks

## Scope and finding

Read-only audit of Longbridge `gpui-pre = 0.3.9`, the pinned approved snapshot. No vendor edits, patch, feature change, build, test, runtime measurement, or GUI acceptance is part of this audit. The workspace pins GPUI without enabling its `profiler` feature (`Cargo.toml:30-33`); that feature is declared as `profiler = ["dep:hdrhistogram"]` in the vendor manifest (`vendor/gpui-pre-0.3.9/Cargo.toml:66`).

The cheapest no-dependency hook available through official GPUI is `Window::on_next_frame`. The method queues a callback and schedules/wakes the platform frame source without making the window dirty (`vendor/gpui-pre-0.3.9/src/window.rs:2709-2719`). Despite the method's after-render documentation, the pinned dispatch loop runs pending callbacks before checking whether drawing is needed (`window.rs:1866-1874`, `1884-1899`). A callback can therefore run without a draw; it does not observe completion of the current draw or presentation. A RustUI-owned wrapper can timestamp callback intervals and collect optional timing for operations it directly invokes. Treat these as **callback intervals** and **RustUI operation durations**; they do not establish frame rate, compositor presentation, GPU time, or latency to pixels.

## What GPUI exposes

- `Window::draw` constructs and assigns a `rendered_frame`; its own documentation explicitly says that showing the scene requires `present` (`vendor/gpui-pre-0.3.9/src/window.rs:3420-3429`). `present` calls `platform_window.draw(scene)` and records platform submission duration only under `feature = "profiler"` (`vendor/gpui-pre-0.3.9/src/window.rs:3603-3618`).
- With `profiler` compiled in, GPUI records `FrameTiming` draw start/end, platform refresh interval/signal when provided, and aggregate request-layout, prepaint, and paint durations (`vendor/gpui-pre-0.3.9/src/profiler.rs:788-821`). It separately defines `PresentTiming` as time spent submitting to the platform, not a display presentation acknowledgment (`vendor/gpui-pre-0.3.9/src/profiler.rs:949-977`). The public collector yields `Draw` and `Present` events (`vendor/gpui-pre-0.3.9/src/profiler.rs:1615-1645`); `FrameDurationSnapshot` provides dirty-to-present, draw duration, and active-animation submitted-present intervals (`vendor/gpui-pre-0.3.9/src/profiler.rs:979-986`).
- Per-view profiler scopes exist for **stateful** `ViewElement`s in request-layout, prepaint, and paint; stateless components take separate paths without those scopes (`vendor/gpui-pre-0.3.9/src/view.rs:344-379`, `382-421`, `423-455`). These are not callback APIs for an application to wrap arbitrary subtrees.
- The feature gates profiler instrumentation and types (`vendor/gpui-pre-0.3.9/src/profiler.rs:1-6`, with `FrameTiming`/`FrameEvent` feature gates at lines 788 and 969-977). It is currently not enabled in the workspace GPUI dependency declaration. Enabling it would be a dependency feature change and is outside this audit's no-change scope.
- No inspected API reports GPU execution time or confirms when a frame reached/presented on the physical display. The macOS backend's platform `draw` delegates to its renderer (`vendor/gpui-pre-macos-0.3.9/src/window.rs:2124-2127`); the inspected public GPUI path does not expose a native presentation-completion hook. Therefore an actual presented-frame hook remains **unknown/unavailable from the inspected public API**.

## Smallest next instrumentation proposal

If later authorized, implement only inside RustUI framework/runtime code and expose it through the public `rustui` API. Do not put native GPUI use in Playground: the canonical Playground must continue to consume only `rustui` (`crates/rustui/src/runtime/window.rs:9-18` shows window ownership in RustUI runtime).

1. For low-overhead cadence diagnostics, schedule a recurring `on_next_frame` callback and retain monotonic timestamps in RustUI-owned state. Label the result `on_next_frame callback interval`; skip FPS and presented-frame claims. Stop scheduling when the consumer/diagnostic session ends.
2. For a bounded CPU breakdown, optionally time only RustUI wrapper operations that it invokes itself (for example, selected top-level render work). Report operation name and elapsed CPU wall time. Do not claim full GPUI layout/prepaint/paint stage coverage: application callbacks do not surround GPUI's internal phases. Existing per-view profiler details would require enabling the GPUI feature and a deliberate public RustUI adapter, which is a separate decision.
3. Keep actual frame interval, compositor presentation latency, GPU duration, and input-to-photon latency unavailable unless a supported presentation acknowledgment is established in a separately approved platform/API audit.

This is an audit proposal only. It authorizes no implementation, GPUI feature change, vendor patch, benchmark, or GUI run.
