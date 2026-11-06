# M1072 — Stage 3 / resolve virtualized list suffixes by active locale

`ListView::localized_suffixes` now resolves per-row message IDs through the
active RenderContext while retaining the existing virtualized string-list
filtering and selection path. The Playground catalog uses it to localize
coverage labels in English, French, and Arabic. The focused locale regression
passed 1/1 and the scoped diff check passed. Native GUI acceptance remains
open.
