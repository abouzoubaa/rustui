# C9: Skeleton adapter

## Public API

- `Skeleton()` creates GPUI Components' animated skeleton placeholder.
- `.size(width, height)` sets explicit logical-pixel dimensions; both values
  must be finite and positive.
- `.corner_radius(radius)` sets the native GPUI rounded style; radius must be
  finite and nonnegative.
- `.secondary()` selects the native secondary skeleton color.
- Skeletons are decorative by default. `.accessibility_label(text)` adds a
  GPUI ProgressIndicator role and nonempty accessible name; `.id(identity)`
  sets an explicit stable ID. Without an explicit ID, a labeled skeleton
  derives one from its label. Identical labeled siblings need distinct IDs.

The shimmer animation and theme colors come directly from pinned GPUI
Components. RustUI adds no timer, animation state, image cache, or loading
fallback. The adapter does not guarantee GPU optimization or shared animation
reuse. Reused/cached components retain the normal RustUI owner lifecycle while
animation scheduling remains GPUI's responsibility.

## Limits and verification boundary

Focused unit tests cover dimension and radius validation. In the M22 native
Review, skeleton bars were visible in the dark third card. Accessibility
inspection found two ProgressIndicator nodes named “Loading item title” and
“Loading item details.” The final unit/integration run passed 77 tests across
12 binaries; see `target/headstart/milestone21-22-tests-final.log`.

This confirms the observed rendering and accessibility tree in that review;
it does not establish animation timing or VoiceOver announcement behavior.
