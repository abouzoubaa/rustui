# M1137 — honor the Web module-script parsing boundary

## Change

The Web packager's HTML-comment scans now ignore `<!--` occurring inside an
executable module script. The HTML parser treats script contents as script
data, so such text must not hide or invalidate the module bootstrap. A
positive fixture covers a JavaScript string containing `<!--` before the
generated import.

## Focused evidence

```text
sh scripts/tests/test_build_playground_web.sh — passed
```

The same focused harness covers prior bootstrap-decoy rejection, the
`nomodule` fallback, and template-literal/backtick handling. Browser runtime,
pointer/keyboard, asset, accessibility, and font/glyph acceptance remain open.
