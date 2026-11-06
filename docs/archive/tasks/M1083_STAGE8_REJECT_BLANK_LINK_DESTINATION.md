# M1083 — Reject blank Link destinations

`Link` destinations must contain at least one non-whitespace character. Reject
blank targets at construction so the public API cannot pass an empty URL to
the native opener.

The focused constructor regression passed 1/1. Native activation and external
link opening remain unverified.
