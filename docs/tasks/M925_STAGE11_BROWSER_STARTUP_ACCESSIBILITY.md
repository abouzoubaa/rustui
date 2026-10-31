# M925 — Stage 11 / expose browser startup busy state accessibly

The Web shell now sets `aria-busy="true"` when its module script begins
JavaScript/Wasm initialization and clears the state on both success and error.
The static body does not begin busy, so a browser with JavaScript disabled
does not leave the document in a permanent busy state.

The focused packaging harness passed:

```sh
sh scripts/tests/test_build_playground_web.sh
```

This validates the packaged shell contract, not a browser's accessibility tree
or runtime announcement. Browser runtime, keyboard, per-control accessibility,
and platform acceptance remain open.
