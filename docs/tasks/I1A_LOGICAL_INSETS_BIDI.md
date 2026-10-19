# I1A Logical Insets and Bidi Isolation

## Scope

Add logical inset resolution for container padding and an opt-in Unicode bidi
isolation helper for dynamic text. This slice uses the active RustUI layout
direction and adds no translation or text-shaping engine.

## Contract

- `LogicalInsets::new(top, bottom, start, end)` accepts finite, nonnegative
  logical pixel values and returns an error otherwise.
- `LogicalInsets::resolve(direction)` maps start/end to left/right while
  preserving top/bottom.
- `Container::padding_edges(insets)` resolves against the current render
  context when lowering.
- `Container::padding(value)` and `padding_edges(value)` are last-setter-wins.
- `BidiIsolation` wraps dynamic text with FSI, LRI, or RLI and a matching PDI.
  The caller chooses when to isolate; text is not modified automatically.

## Verification

Unit coverage checks LTR/RTL inset mapping, rejection of negative and
non-finite values, and isolate controls around Unicode and empty strings.

## Limits

These controls do not translate content, select locale-specific fonts, or
change GPUI's Unicode shaping behavior. Logical padding is currently exposed
on `Container`; it does not add directional padding methods to other views.
