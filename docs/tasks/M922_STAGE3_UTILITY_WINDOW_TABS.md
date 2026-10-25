# M922 — Stage 3 / mirror Utility Window reference categories

The catalog's Utility Window preview now renders the Utility Panel title and
five named category tabs from the SVG reference: Color Wheel, Sliders,
Swatches, Images, and Crayons. Color Wheel is selected initially. The specimen
uses the public RustUI `TabView` API and exposes the category through an
accessibility label. The catalog note now describes this composition and its
remaining native acceptance work.

The focused regression passed 1/1 (139 filtered):

```sh
./scripts/cargo-headstart test -p rustui-playground utility_window_gallery_matches_the_reference_tab_groups --locked --offline
```

Native visual appearance, keyboard interaction, and accessibility acceptance
remain open.
