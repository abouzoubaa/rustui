# M1179 — Stage 6 / verify explicit Adlam script direction

The locale resolver already classifies the explicit Adlam script subtag
(`Adlm`) as right-to-left in the shared worktree. Its explicit-script
regression did not cover that entry, so `und-Adlm` could have regressed without
failing the test.

Added `und-Adlm` to `explicit_script_overrides_language_direction`. The focused
test passed 1/1, and scoped `git diff --check` passed:

```sh
./scripts/cargo-headstart test -p rustui-i18n explicit_script_overrides_language_direction
```

This adds resolver coverage only. Full CLDR behavior and native RTL/platform
acceptance remain open.
