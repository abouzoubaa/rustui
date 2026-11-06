# M69: conditional stack macro grammar

## Contract

`VStack!` and `HStack!` accept `if`, `if`/`else`, and `else if` branches alongside
their existing adjacent child expressions:

```rust,ignore
VStack! {
    Text("Always present")
    if self.editing() {
        Keyed("editor", TextEditor("Edit", self.text_binding()).id("body"))
        Button("Save", save)
    } else if self.pending() {
        Text("Waiting")
    } else {
        HStack! { Text("Read only") Button("Edit", edit) }
    }
}.spacing(8.0)
```

Each branch body contains zero or more stack children. Branches may nest and may
have different child types or counts. A missing `else` contributes no children
when its condition is false. Conditions run while the component describes its
body; each reached condition is evaluated once with ordinary Rust short-circuit
branch selection. Unselected branch child constructors are not evaluated.

Existing constructor calls, method calls on previously constructed views,
optional children such as `visible.then(|| child())`, nested stack macros,
chained builders, and optional commas/semicolons remain supported. A bare method
name in an unqualified two-argument `Button(label, handler)` is still rewritten
to `self.action(Self::handler)`, including inside conditional branches. Explicit
actions and qualified constructor/handler paths retain their existing meaning.
Nested stack macros expand their own children and action shorthand.

This slice adds no `for`, `match`, arbitrary statement/block, tuple, or bare-value
child grammar. Use existing `.children(...)`, `Option<V>`, `AnyView`, `Keyed`, and
builder APIs for those composition needs. Constructor arguments retain ordinary
Rust expression syntax; these exclusions concern the stack's child list.

## Existing runtime and identity

The macro expands a branch to the existing `Group::new().child(...).group(...)`
composition and appends it through the receiving stack's `.group(...)`. Group
flattens children into their parent stack. It adds no native layout node,
retention scope, reactive runtime, callback owner, or branch identity.

Callers must supply explicit stable `Keyed` identities for stateful dynamic
children. IDs must remain unique across all simultaneously selected flattened
siblings, including children outside a conditional. Different alternatives may
reuse a key deliberately; existing owner/type reconciliation rules still apply.
The macro does not create positional IDs or preserve omitted native entities.
Removing a child follows normal release semantics; restoring it remounts its
native state. Component-owned binding values may survive that removal because
their actual owner remains mounted.

## Implementation and regression evidence

- `crates/rustui-macros/src/view_builder.rs` parses child lists and conditionals
  using the existing Syn dependency and emits ordinary stack/Group builders.
- Parser/expansion unit tests use `proc_macro2` tokens, so they do not need an
  active compiler proc-macro context. Every successful expansion is parsed back
  into a Rust expression before its structure is checked.
- Ten focused tests cover the original adjacent/separated/chained grammar;
  existing-value method calls and optional child closures; absent `else`;
  heterogeneous multi-child alternatives and horizontal stacks; nested branches
  and explicit keys; shorthand versus explicit/qualified actions; single
  condition emission and empty branches; malformed conditionals; and rejected
  `for`/`match`/bare/block/tuple child syntax.
- Adjacent parenthesized optional children keep the original DSL boundary:
  `controls.spacing(8.0) (condition).then(...)` becomes two children, rather than
  a function call on the first view. This boundary is also checked inside a
  conditional branch.
- The existing-value regression cases match forms already used by Playground's
  retained story controls and pending-task release story. Those forms must keep
  compiling as this grammar expands.

## Playground and acceptance

`stories/conditional_gallery.rs::demo()` supplies a public-API component for
root story registration. It exercises all three alternatives with heterogeneous
content, nested horizontal/vertical macros, multi-child branches, conditional
action shorthand, an optional keyed editor, two keyed editors reordered through
alternative branches, and a keyed tail after the dynamic content. The editors
deliberately reuse the same inner input ID under distinct explicit keys.

Root compilation of the registered gallery is the real macro type-checking
exercise. Native acceptance should edit distinct A/B/tail values, reverse the
editors, switch branches and update the parent, then confirm independent values
and stable native editing state for continuously mounted keyed children. Omit
and restore the optional editor, checking preserved owner text and remounted
native state. Confirm each visible branch action reaches its actual owner and
unselected actions are absent. Repeat under both locale directions; branch
selection itself introduces no direction policy.

The final M69–M71 workspace suite passed 236 tests across 12 binaries, and the build succeeded with all three pinned GPUI dependencies fresh. The locked/offline all-targets check exited 0. Bounded native acceptance is complete and recorded in `target/headstart/milestone69-conditional-stacks-gui.json`. It observed branch selection, keyed editor reorder, action dispatch, parent updates, and optional-child omission/restoration. Stateful children require explicit `Keyed` identity. RTL, full keyboard/accessibility speech, cursor/undo, release callbacks, and performance remain unverified; `for`/`match` syntax is outside this milestone..

### M551 generic handler paths

The Button shorthand transform applies only to a bare handler identifier. An unqualified turbofish handler such as `save::<Commit>` retains its explicit type arguments and is not rewritten to `self.action(...)`. The focused explicit/qualified-handler regression passed 1/1. Evidence: `target/headstart/evidence/m551-explicit-generic-button-action-test-20261021.log`.
