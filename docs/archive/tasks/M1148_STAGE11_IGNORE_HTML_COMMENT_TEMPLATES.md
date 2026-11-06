# M1148 — Stage 11: ignore HTML comment text in Web bootstrap parsing

The Web package guard must only track JavaScript template literals inside an
executable module script. A backtick in an HTML comment previously left its
template state open, causing a valid generated-module import and initializer to
be rejected.

The focused regression failed before the fix with `Web browser shell does not
load rustui_playground.js` and passed after the guard limited template tracking
to module-script content:

```sh
sh scripts/tests/test_build_playground_web.sh
bash -n scripts/build-playground-web.sh
git diff --check -- scripts/build-playground-web.sh scripts/tests/test_build_playground_web.sh
```

These checks cover the package guard only. Browser runtime, input, accessibility,
and font/glyph acceptance remain open.
