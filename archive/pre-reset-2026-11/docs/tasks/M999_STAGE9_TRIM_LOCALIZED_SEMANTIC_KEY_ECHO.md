# M999 — omit trimmed localized key echoes from semantic text

`SemanticText::resolve` now compares an unresolved localized value with the
trimmed message key. A whitespace-padded key echo therefore remains omitted
from the accessibility tree instead of becoming a user-facing label. The
existing focused regression now covers a padded key, and passed 1/1:

```text
./scripts/cargo-headstart test -p rustui localized_semantic_text_omits_key_echoes_and_blank_translations --locked
```

Native screen-reader, visual, keyboard, and cross-platform acceptance remain
open.
