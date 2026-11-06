# O5: Inline alert

`components::overlay::alert::Alert` adapts the pinned GPUI Components `Alert`
element. It keeps the upstream semantic variants, optional title, close button,
and themed presentation. Visibility is controlled through an owner `Binding<bool>`;
closing writes `false` to that binding. Literal and catalog-backed title/message
text are supported.

## Gallery example

```rust
#[component]
struct AlertGallery {
    #[state(true)]
    visible: bool,
}

impl View for AlertGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .child(
                Alert(
                    "saved-alert",
                    "Your changes were saved.",
                    self.visible_binding(),
                )
                .title("Saved")
                .variant(AlertVariant::Success),
            )
            .child(Button("Show alert", self.action(Self::show_alert)))
    }
}

impl AlertGallery {
    fn show_alert(&mut self) {
        self.visible = true;
    }
}
```

The snippet describes the standalone gallery behavior; it is not yet connected
to a Playground route or catalogue entry.

## Boundaries

- The wrapper delegates the alert surface, semantic variant, title, and message
  to the pinned GPUI Components `Alert`. RustUI adds a localized, focusable
  `Close` button in the surrounding group and leaves the upstream
  unlabeled, non-focusable close element disabled. No overlay, animation, or
  reactive runtime is added.
- Localization resolves existing message keys through `RenderContext`; without
  an active catalog, the key itself is shown, matching other RustUI adapters.
- The upstream alert requests named icons for its variants. The app still lacks
  the approved upstream icon asset source, so variant icon pixels may be blank.
  RustUI's close action uses localized text and does not depend on the missing
  close icon asset. This implementation adds no assets or dependency and does
  not claim complete visual acceptance; keyboard focus and activation were not
  verified in the M41 native review.
- Unit tests cover builder state, variant mapping, localization selection, and
  close-state mutation. Bounded native GUI observations are recorded in
  `target/headstart/milestone41-alert-gui.json`: Close was named in the native
  accessibility tree and removed the alert when activated; French changed its
  name to Fermer; external Hide/Show removed/restored the alert; Success appeared
  green, Warning amber, and Error red. Default/Info appearance, icon pixels,
  keyboard focus/activation, and full alert screen-reader announcement remain
  unverified.
