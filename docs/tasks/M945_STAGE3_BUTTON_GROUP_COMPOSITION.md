# M945 — Stage 3 / represent Button Group with public controls

The Button Group catalog row now opens a small grouped composition made from
three public `Button` controls inside an `HStack`. Each button updates the
visible last-action value. This is a representative composition; it does not
claim a dedicated RustUI ButtonGroup API or recreate the reference SF Symbol
artwork.

The focused button-group filter passed 2/2 (the story state regression and
catalog route regression):

```sh
./scripts/cargo-headstart test -p rustui-playground --lib \
  button_group --locked --offline
```

An additional source regression dispatches the generated Previous, Next, and
More actions and verifies each notifies and updates the visible last-action
value. Focused verification passed 1/1 (185 filtered):

```sh
CARGO_PROFILE_TEST_DEBUG=0 ./scripts/cargo-headstart test -p rustui-playground \
  --locked --offline --lib button_group_generated_actions_update_last_action_feedback
```

Native visual fidelity, keyboard interaction, and accessibility acceptance
remain open.
