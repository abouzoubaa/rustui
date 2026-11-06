# Canvas basic vector drawing

`Canvas` is a bounded RustUI drawing leaf backed by GPUI's native canvas element. Playground's Canvas story uses only the public RustUI API.

```rust
let mut drawing = CanvasScene::builder();
drawing.fill_rect(
    CanvasRect::new(0.0, 0.0, 120.0, 80.0)?,
    Hsla { h: 0.58, s: 0.8, l: 0.5, a: 1.0 },
)?;
let scene = drawing.build();
let view = Canvas(CanvasSize::new(120.0, 80.0)?, scene.clone());
```

The fixed canvas size is in logical pixels, finite, and nonnegative; zero supports a collapsed parent. Local coordinates may extend outside the canvas and are clipped to its rectangular bounds. Rectangles require positive extents, stroke widths must be positive and fit inside the stroked rectangle, and lines require distinct endpoints. Coordinates, derived rectangle edges, and line corners must remain representable as finite `f32` values. Colors use RustUI's existing `Hsla` facade with finite channels in `0..=1`. The scene preserves command paint order. Invalid user input returns `CanvasError` while building the scene. A rare native bound offset that cannot be represented is reported once per scene to stderr and skips that paint batch.

`CanvasScene` clones share immutable commands. Store a common scene in component state when several canvases use the same geometry; rebuild and replace it through normal component updates when drawing state changes. `GeometryReader` can supply a responsive size, with its own bounded-parent contract. Canvas owns no scheduler or rendering runtime: GPUI handles layout, paint, scale, opacity, and rectangular clipping. The current stroke line has butt ends and is painted as a native path made from two triangles.

This basic slice has filled rectangles, rectangle borders, and straight lines. It has no arbitrary path builder, circles, text or image drawing, per-shape hit tests, or independent animation. The Canvas leaf has no per-primitive accessibility representation; wrap an illustration in `Semantics` with a summary and provide equivalent information in regular RustUI text or controls. The Playground story does both.

Validation requires a native build and visual check of the Playground story, including its clip toggle, plus accessibility inspection of the summary and text. Source-level unit checks cover parameter validation, shared scenes, and native paint preparation; they do not establish rendered pixels or accessibility output.

## M61 bounded native acceptance

The M61 native review is recorded in
`target/headstart/milestone61-canvas-gui.json`, bundle SHA-256
`ba2979d96f69ed94c6dc1edd1fba285c65e529d452f6154c5016acaf79f5ebce`. The
review observed the filled blue rectangle, amber outlined rectangle and
diagonal line; the same scene in a smaller canvas clipped to its own bounds;
the clip toggle moving the blue rectangle across the left edge while clipping
cleanly in both canvases; descriptive group labels in the accessibility tree;
and the outer Playground preview scrolling to the gallery.

This accepts only that basic drawing, clipping, and group-label slice. Exact
pixels, a scale-factor matrix, screen-reader speech, arbitrary paths, text/image
drawing, per-shape accessibility/hit testing, and performance remain unverified.
The repaired suite passed 190 tests with 0 failures across 12 binaries in
`target/headstart/milestone61-62-tests-repaired.log`. The build passed and all
three pinned dependencies were fresh according to
`target/headstart/milestone61-62-reuse-report.json`; the locked/offline
all-targets check passed in
`target/headstart/milestone61-62-all-targets-check.log`.
