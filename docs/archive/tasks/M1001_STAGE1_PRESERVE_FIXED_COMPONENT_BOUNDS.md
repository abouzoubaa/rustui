# M1001 — preserve fixed component bounds under flex pressure

Fixed-size cached child styles and their uncached fallback now set
`flex_shrink` to zero. The component's documented width and height therefore
remain fixed when the parent flex layout is constrained.

The focused tests passed 3/3:

```text
./scripts/cargo-headstart test -p rustui --lib fixed_bounds --locked --offline
```

This is style-level evidence; broader native visual and cross-platform layout
acceptance remain open.
