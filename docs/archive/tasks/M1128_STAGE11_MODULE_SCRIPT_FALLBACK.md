# M1128 — explain unsupported module scripts at Web startup

## Change

The Web shell adds a `nomodule` fallback message to its existing `role=status`
startup region. Browsers that support JavaScript but not module scripts can
now explain why the RustUI Playground did not start. The Web packaging test
asserts that the fallback is included in the packaged page.

## Focused evidence

```text
sh scripts/tests/test_build_playground_web.sh — passed
```

This confirms packaging and the fallback text only. Browser runtime, pointer
and keyboard behavior, asset loading, accessibility, and font/glyph acceptance
remain open.
