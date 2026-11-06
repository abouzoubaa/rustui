# M726 — Stage 2 stack gap span

HStack and VStack accept finite, nonnegative spacing. With three or more
children, a very large finite spacing could still make the combined span across
gaps nonfinite before GPUI received the layout. Both builders now validate the
current child gap span when spacing or child count changes, including children
inserted through `Group`.

The focused regressions cover overflow when spacing is set before children and
when spacing is changed after children, for both HStack and VStack. The locked,
offline Headstart check passed 2/2; output is preserved at
`target/headstart/evidence/m726-stage2-stack-gap-span-tests.log`:

```text
./scripts/cargo-headstart test -p rustui --lib total_spacing_span_must_remain_finite --locked --offline
```

This is a source-level numeric guard. It does not claim GUI layout acceptance.
