# M329 — Stage 9 / D1c CodeEditor preview specimen

The compiled preview gallery exposes RustUI's public `CodeEditor` through one
fixed plain-text specimen. It uses the public RustUI API, fixes the language to
`text`, and keeps editing read-only. Its only descriptor property is the
bounded `value` string (maximum 2,000 characters); the descriptor has no events
or actions. No language server, syntax grammar, code execution, or source
editing behavior is part of this specimen.

Acceptance evidence:

- Focused regression verifies the one-property schema, that the controlled
  CodeEditor binding reads and writes the declared `value`, empty action/event
  maps, round-trip at the schema limit, and rejection above the limit.
- The Playground passes an isolated locked/offline all-targets check.
- Scoped rustfmt and `git diff --check` pass.

This is source and compile evidence only. It does not claim GUI or native
interaction acceptance.
