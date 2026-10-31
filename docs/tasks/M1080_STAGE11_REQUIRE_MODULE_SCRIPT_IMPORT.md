# M1080 — Require a module script in the Web shell

The Stage 11 Web packager must find the generated `rustui_playground.js`
import inside an executable `<script type="module">`. An import-like line in
an inert script block must not qualify the browser shell for packaging.

The focused packaging harness passed, covering rejection, preservation of the
existing bundle, and staging cleanup. Browser runtime, keyboard, text-entry,
asset, font/glyph, and accessibility acceptance remain separate open gates.
The pending GPUI keyboard patch approval was not changed or applied.
