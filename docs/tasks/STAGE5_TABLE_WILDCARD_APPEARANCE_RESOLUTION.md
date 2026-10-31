# Stage 5 Table header wildcard appearance resolution

Component style selectors use `any` to match any runtime value, and the shared
component resolver applies that wildcard with the same specificity rules as
other selectors. Table header activation previously bypassed that behavior by
searching for an exact `light` or `dark` rule before resolving colors.

Table header colors and menu backgrounds now resolve directly against the active
appearance query. Regressions remove the exact dark rule, change the remaining
light rule to `any`, validate each theme, and verify both appearances resolve
the same result. The focused regressions passed 2/2, and the locked/offline
RustUI library check passed. Scoped diff checks passed; the file-wide rustfmt
check reports pre-existing import ordering. Native visual and cross-platform
theme acceptance remain open.
