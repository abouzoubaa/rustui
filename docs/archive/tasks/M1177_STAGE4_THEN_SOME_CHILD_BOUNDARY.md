# M1177 — Stage 4 / preserve adjacent `then_some` optional children

The stack-child parser recognized a parenthesized `.then(...)` expression as
an adjacent optional child, but did not recognize `.then_some(...)`. In an
expression such as `Text("before") (visible).then_some(Text("optional"))
Text("after")`, the parser could absorb the optional child into the preceding
constructor call.

The boundary recognizer now accepts both method names. A focused expansion
regression asserts the before, optional, and after expressions remain three
separate `.child(...)` calls in order. The focused test passed 1/1:

```sh
./scripts/cargo-headstart test -p rustui-macros --locked --offline then_some_optional_child_keeps_adjacent_constructor_siblings
```

No broad macro, workspace, or native GUI acceptance was run.
