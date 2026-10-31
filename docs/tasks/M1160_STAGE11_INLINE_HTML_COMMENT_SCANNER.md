# M1160 — Stage 11: preserve Web scripts after inline HTML comments

The Web package guard previously skipped the rest of a physical line whenever
it encountered an HTML comment. A valid module script after a same-line comment
was therefore invisible to both bootstrap scans and the packager rejected the
shell.

Both scanner passes now strip HTML comments while preserving the surrounding
markup. The focused fixture places an executable module script after an inline
comment. It failed before the fix with `Web browser shell does not load
rustui_playground.js` and passes afterward.

Focused checks passed:

```sh
sh scripts/tests/test_build_playground_web.sh
bash -n scripts/build-playground-web.sh
sh -n scripts/tests/test_build_playground_web.sh
```

This validates packaging guard behavior only. Browser runtime, input,
accessibility, and font/glyph acceptance remain open.
