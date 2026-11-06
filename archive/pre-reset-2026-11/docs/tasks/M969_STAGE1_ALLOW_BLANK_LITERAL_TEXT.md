# M969 — Allow blank literal Text content

Stage 1 / basic Text behavior.

The `Text` view accepts empty and whitespace-only literal strings. The
renderer omits an accessible label when the resolved visible content is blank.
Blank localized message identifiers continue to be rejected.

The focused regression passed 1/1. Rust formatting and scoped whitespace
checks passed.
