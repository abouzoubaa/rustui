# M1068 — Stage 4 / ignore cfg-disabled fields in accessor collision checks

The macro collision precheck now considers only state fields without
`cfg`/`cfg_attr` conditions. Rust handles potential collisions among
conditionally generated accessors after applying the active configuration.
The focused regression passed 1/1; formatting and scoped whitespace checks
passed.
