# M1273 — Stage 4: type-check turbofish optional children through `HStack!`

Primary checkout comparison at `cb165c36b6d168895fbcfbd45140889627d54b24`
found no public `HStack!` integration test for M1211's turbofish `.then` and
`.then_some` optional-child boundary. The current worktree has a focused
integration test compiling both forms between adjacent `Text` children.

This adds public horizontal macro type-check evidence for the existing
M1211 parser contract. It does not change parser behavior or the public API.

Verification: `./scripts/cargo-headstart test --profile headstart -p rustui
--test hstack_macro_turbofish_optional --locked --offline` passed 1/1. Cargo
emitted the existing future-incompatibility warning for `block v0.1.6`.

Broader macro and Stage 4 acceptance remain open.

Primary checkout recheck: `./scripts/cargo-headstart test --profile headstart -p rustui --test hstack_macro_turbofish_optional --locked --offline` exited 0 (the single integration test passed).
