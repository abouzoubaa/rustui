# M970 — Toast live announcements

Stage 9 / accessibility announcements.

Toast notification content now exposes a polite `Status` live region for
informational, success, and warning notifications. Error notifications expose
an assertive `Alert` live region. The announcement value combines a nonblank
title and message, falling back to whichever is present.

The focused Toast test module passed 7/7. Rust formatting and scoped whitespace
checks passed. Native accessibility-tree and screen-reader announcement
acceptance remain open.
