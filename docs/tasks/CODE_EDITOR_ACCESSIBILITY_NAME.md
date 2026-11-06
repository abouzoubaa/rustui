# CodeEditor accessible-name validation

`CodeEditor` keeps its accessible name optional. When callers provide an
`accessibility_label(...)` override, it must contain at least one
non-whitespace character. The builder checks the trimmed view and retains
nonblank input exactly as supplied.

Focused source tests cover the absent default, whitespace-only rejection, and
verbatim preservation of a nonblank override. These tests do not establish
native accessibility-tree, screen-reader, or cross-platform acceptance.
