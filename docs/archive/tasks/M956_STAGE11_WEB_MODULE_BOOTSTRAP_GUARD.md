# M956 — Stage 11 / require the browser shell to load the generated module

The Web packaging script now rejects a nonempty shell that omits the generated
`rustui_playground.js` import. The focused fake-tool harness verifies that the
inert shell is rejected, the previous bundle is preserved, and staging is
cleaned up.

This guards package readiness only. It does not establish browser startup,
rendering, input, asset loading, accessibility, or broader platform
acceptance.
