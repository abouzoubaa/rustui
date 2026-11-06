# M1006 — Stage 11 / reject block-commented browser imports

The Web package guard now ignores candidate module imports inside JavaScript
block comments. The focused fake-tool harness verifies that a nonempty shell
with a block-commented import is rejected, the previous bundle is preserved,
and staging is cleaned up.

This closes a package-readiness gap only. Browser startup, rendering, input,
asset loading, accessibility, and platform acceptance remain open.
