# M873 — Stage 4 conditional Default bounds

## Change

The component macro now handles `cfg` and `cfg_attr` on defaulted fields
without attaching an unconditional `T: Default` bound for a field that is
removed by configuration. A private type alias inside an anonymous const
selects the field type when the field is present and `PhantomData<T>` when it
is absent. This uses stable Rust syntax and does not change the public macro
API.

## Focused evidence

- The compile fixture covers direct `cfg`, `cfg_attr`, nested `cfg_attr`, and
  the enabled-field `T: Default` requirement.
- `./scripts/cargo-headstart test -p rustui-macros --test conditional_default --locked --offline` passed: 1 passed.
- An independent downstream `rustc` consumer probe confirmed the generated
  public `Default` implementation remains usable.
- Scoped formatting and `git diff --check` passed.

## Acceptance boundary

This closes conditional generic-bound behavior for generated component
`Default` implementations. Broader macro integration and workspace acceptance
remain separate.
