# M820 Stage 5: Checkbox and Radio artwork geometry

Checkbox and Radio renderers read vector artwork as an untransformed rectangle
using the path commands `M H V H V Z`, six finite coordinates, a closed path,
and positive width and height. They require a six-value identity transform.
Theme validation now enforces those runtime assumptions before resolving the
theme, so malformed data is rejected rather than triggering renderer
assertions.

Focused regression:

```text
rtk proxy ./scripts/cargo-headstart test -p rustui theme::schema::tests::checkbox_and_radio_themes --lib --locked --offline
2 passed; 0 failed
```

Native visual rendering, accessibility, and cross-platform theme acceptance
remain open.
