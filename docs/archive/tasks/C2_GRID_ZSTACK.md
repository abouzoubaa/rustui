# C2 Grid and ZStack

## Grid

`Grid::new(NonZeroU16)` creates a fixed-column grid backed by GPUI's native
Taffy grid layout. `spacing` applies an equal row and column gap; `fill` and
`grow` match the existing stack layout behavior. Children are placed in
row-major order, with each row's columns mirrored for RTL locales.

GPUI grid line placement uses signed 16-bit lines. Grid column counts and row
positions therefore must fit that upstream range. Tracks are uniform and
rows size to their content. Custom tracks, cell spans, auto-fit, and
virtualization are outside this API.

The Playground example uses a small, fixed set of six items so it stays within
the native screenshot area. RTL mirroring follows the active Playground locale;
the grid has no separate direction override.

## ZStack

`ZStack` places each child in the same GPUI grid cell and uses GPUI flex layout
for explicit two-axis alignment. `ZStackAlignment` combines the existing
`HorizontalAlignment` and `VerticalAlignment` values. The default alignment is
center/center. Children are supplied back-to-front, so later children paint
above earlier children.

Dynamic children should use `Keyed` to retain identity across reorderings, as
they do in other RustUI stacks. The layout container itself does not own
component state.

## Verification

Focused unit tests cover row-major and RTL placement, grid-line overflow,
invalid spacing, default overlay alignment, and logical horizontal flow.

## Executed native acceptance

The M17 Grid Review build showed two rows with three columns: English rows 01/02/03 and 04/05/06. Selecting Arabic mirrored each row to 03/02/01 and 06/05/04 visually from left to right. The centered Front layer visibly painted above the Back layer. Exact binary hash and observations: `target/headstart/milestone17-grid-overlay-gui.json`. Only centered overlay alignment was visually accepted; other alignments still need GUI coverage. The initial demo mistakenly applied Container sizing methods to VStack; two failed builds are retained and final accepted build exited 0.
