# Stage 3 — additional Text style reference samples

The Primitive gallery keeps the three M929 samples and adds five further
reference size/weight pairs supported by the public `Text` API:

- Large Title Regular — 26 pt
- Title1 Regular — 22 pt
- Title2 Regular — 17 pt
- Title3 Regular — 15 pt
- Headline Bold — 13 pt

The focused gallery regression asserts the complete eight-row sample list.
The SVGs also contain Inter and variable weights such as 510, 590, and 860;
the public API does not set a primary font family or arbitrary font weights,
so those styles remain outside this sample set. Native visual and accessibility
acceptance remain open.

The focused gallery regression passed 1/1, and scoped formatting/diff checks
passed:

```text
./scripts/cargo-headstart test -p rustui-playground --lib stories::primitive_gallery::tests::text_gallery_matches_reference_size_and_weight_samples --locked --offline
```
