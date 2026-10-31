# Localized navigation fallbacks

NavigationSplit and Toolbar keep a nonblank constructor label when a nonblank
localization key resolves to whitespace. NavigationSplit passes the retained
label to Sidebar and uses it for the selected detail region. A localized split
tree name preserves an earlier literal accessibility name or falls back to
`Navigation`. Toolbar keeps its action label as the visible and accessible
button name. Nonblank resolved translations still take precedence; blank
localization keys remain rejected.

## Verification

- `blank_localized`: 5 passed. Log:
  `target/headstart/evidence/localized-navigation-fallback-tests-20261020.log`.
- Navigation module: 28 passed. Log:
  `target/headstart/evidence/navigation-module-tests-20261020.log`.
- Menu module: 12 passed, as part of the menu fallback task.
- Rustfmt and scoped `git diff --check` passed for both changed Rust files.

These source-level tests do not establish native GUI accessibility acceptance.

## M500 — visible fallbacks for unresolved navigation messages
Sidebar row labels and NavigationSplit detail/action labels now fall back to their authored visible text if localization is missing or resolves to the message key itself. Focused tests cover blank and unresolved message results. Navigation unit tests passed 53/53 with the locked/offline Headstart toolchain; rustfmt and scoped diff checks passed. This is source-level behavior evidence, not GUI, VoiceOver, or runtime locale-switching acceptance. Evidence: `target/headstart/evidence/m500-navigation-label-fallback-tests-20261021.log`.
