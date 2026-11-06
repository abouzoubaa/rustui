# M47 — Container shadow

`Container(child).shadow(offset_x, offset_y, blur_radius, spread_radius, color)`
adds one native GPUI outer `BoxShadow`. A later call replaces the previous shadow.
Offsets and spread must be finite; blur must be finite and nonnegative. Signed
spread is supported. No shadow is configured by default.

The existing Container lowers this style directly; no additional rendering,
caching, or animation runtime is involved. This does not define Card semantics
or invent theme shadow defaults.

## Executed evidence

- Workspace unit/integration suite: 135 passed, zero failed across 12 binaries,
  `target/headstart/milestone46-50-tests.log`.
- Native Menu gallery in Light appearance showed an outer diffuse shadow on
  the configured white surface, alongside an unshadowed bordered surface.
- Reviewed binary and bounded observations:
  `target/headstart/milestone47-container-shadow-gui.json`.

The comparison cards have different labels and widths. Exact pixel radii,
clipping, overlapping/translucent compositions, dark appearance, and other
platforms have not been accepted by this observation.
