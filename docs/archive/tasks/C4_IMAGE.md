# C4: Image adapter

## Public API

- `Image(source)` returns `ImageView`. A `&str`, `String`, or `SharedString`
  names an embedded application asset. Pass `PathBuf` or `&Path` for a
  filesystem image.
- `.size(width, height)` sets explicit logical-pixel bounds and requires both
  values to be finite and positive. Without it, GPUI uses the decoded image's
  intrinsic dimensions.
- `.fit(ImageFit)` maps `Contain`, `Cover`, `Fill`, `ScaleDown`, and `None`
  directly to GPUI `ObjectFit` behavior.
- `.accessibility_label(text)` gives a content image the GPUI Image role and
  accessible name. Omit it for decorative images.
- `.id(identity)` sets the stable GPUI accessibility element ID. Labeled images
  derive one from their source and label by default; repeated siblings with the
  same source and label need distinct explicit IDs.

Loading and caching use GPUI's `img` element and application image cache. Asset
names resolve through the app's configured GPUI asset source; file paths resolve
through GPUI's built-in resource loader. The adapter does not create a second
cache or loading runtime.

## Limits and verification boundary

The adapter accepts embedded asset names and filesystem paths only. It does not
interpret strings as URLs, decode bytes itself, add loading/error placeholders,
or map SF Symbols. Image formats and load failures follow the pinned GPUI
loader. Focused unit tests cover source mapping and dimension validation.
Compile, runtime loading, visual fit, and accessibility-tree behavior remain to
be verified by the root task; no build or GUI check was run here.
