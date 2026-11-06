# M1246 — Stage 11 / recognize valid HTML module-script attributes

The Web package preflight recognizes case-insensitive `SCRIPT` tag and `TYPE` attribute names, and both single- and double-quoted `module` values. It requires an actual `type` attribute, so `data-type='module'` does not make a classic script look executable.

The focused packaging regression passed, including acceptance of `<SCRIPT defer TYPE = 'module'>` and rejection of the `data-type` lookalike. No browser bundle was rebuilt.

This is packaging-source validation only. Fresh-browser interaction, keyboard, accessibility-tree and screen-reader output, and complete font/glyph acceptance remain open.
