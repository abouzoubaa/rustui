# M1029 — Reject HTML-commented Web imports

The Web packaging guard could count a browser bootstrap import written inside
an HTML comment. That allowed a package with no active import to pass this
source-level check.

The guard now ignores import-looking lines while inside HTML comments. The
focused shell harness reproduced the false pass before the fix and passed
afterward:

```text
sh scripts/tests/test_build_playground_web.sh
```

The regression also checks that a rejected package preserves the previous
bundle and removes temporary staging output. Browser startup, rendering,
interaction, assets, accessibility, and platform acceptance remain open.
