# M1095 — Stage 0: describe the Playground asset provider accurately

The architecture guide now records that Playground uses RustUI's public API
and the upstream `gpui-kit-assets` asset provider. RustUI's
`with_embedded_asset` helper supplies the Playground sample image. The
application does not call GPUI or GPUI Components APIs directly.

A focused static source/manifest check passed. This corrects the documented
dependency boundary; it does not change the runtime architecture.
