# C1A: Vertical ScrollView baseline

## Status

Implemented: bounded vertical scrolling baseline using the pinned GPUI native div overflow behavior.

## Public API

- `ScrollView(content)` constructs a `ScrollViewView`.
- `.id(identity)` assigns the viewport identity. The default is `scroll-view`; callers should supply unique IDs for sibling or dynamically created scroll views.
- `.height(points)` sets an explicit finite positive height in GPUI points. Non-finite or non-positive values panic with a clear message.
- By default, the viewport fills available width and height. An explicit height overrides the fill height and disables flex growth.

## Behavior and limits

The viewport uses GPUI's `overflow_y_scroll()` and clips/scrolls vertically. Its child is lowered through the same RustUI `RenderContext`; the content wrapper is non-shrinking so content can extend past the viewport. No horizontal scrolling, custom scrollbar, or RustUI-owned scroll handle is exposed. A useful default size therefore depends on a bounded parent/window layout.

## Evidence

The pinned source snapshot's `vendor/gpui-pre-0.3.9/examples/scrollable.rs` uses `div().overflow_scroll()`. `vendor/gpui-pre-0.3.9/src/elements/div.rs` provides `overflow_y_scroll()` and optional `track_scroll()` APIs. The baseline uses native overflow only.

## Combined integration evidence

The final locked all-target workspace check passed with exit 0 (1.87 seconds),
recorded in `target/headstart/roadmap-milestone6-check-final.{jsonl,log}`.
Non-recursive rustfmt checking passed. Required GPUI dependency check artifacts
were all fresh. No tests or GUI/accessibility checks ran; native scrolling remains
an implemented contract awaiting interactive acceptance.
