//! A themed panel surface with conventional padding and elevation.

use std::rc::Rc;

use gpui::AnyElement;
use rustui_core::Identity;

use crate::{
    layout::container::{Container, Surface},
    modifiers::shadow::{Shadow, ShadowElevation},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// Creates a panel card using active theme spacing and radius, with Medium
/// shadow elevation. Explicit `.padding()` and `.radius()` values override the
/// theme defaults.
///
/// `Card` supplies a visual surface and layout padding. It does not clip its
/// child, add a focus or accessibility role, or create a component owner.
/// `id` must be nonempty and scopes the card's retained child namespace.
#[allow(non_snake_case)]
pub fn Card<V: View>(id: impl Into<Identity>, child: V) -> CardView<V> {
    let id = id.into();
    assert!(!id.as_str().is_empty(), "Card ID must not be empty");
    CardView {
        id,
        child: Rc::new(child),
        padding: None,
        radius: None,
        elevation: ShadowElevation::Medium,
        border: false,
    }
}

/// Configurable themed card.
pub struct CardView<V: View> {
    id: Identity,
    child: Rc<V>,
    padding: Option<f32>,
    radius: Option<f32>,
    elevation: ShadowElevation,
    border: bool,
}

impl<V: View> CardView<V> {
    /// Sets equal logical-pixel padding on all sides.
    ///
    /// Panics if `padding` is negative or nonfinite.
    pub fn padding(mut self, padding: f32) -> Self {
        assert!(
            padding.is_finite() && padding >= 0.0,
            "Card padding must be finite and nonnegative"
        );
        self.padding = Some(padding);
        self
    }

    /// Sets the logical-pixel corner radius.
    ///
    /// Panics if `radius` is negative or nonfinite. The radius does not clip
    /// the child.
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "Card radius must be finite and nonnegative"
        );
        self.radius = Some(radius);
        self
    }

    /// Selects one of the active theme's native shadow elevations.
    pub fn elevation(mut self, elevation: ShadowElevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Adds or removes a one-pixel border using the active theme's border color.
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }
}

impl<V: View> View for CardView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let (theme_padding, theme_radius) = context.card_defaults().unwrap_or((16.0, 10.0));
        let padding = self.padding.unwrap_or(theme_padding);
        let radius = self.radius.unwrap_or(theme_radius);
        context.with_element_scope(RetentionScope::Card(self.id.clone()), |context| {
            let surface = Container(CardChild(self.child.clone()))
                .padding(padding)
                .radius(radius)
                .background(Surface::Panel);
            let surface = if self.border {
                surface.border()
            } else {
                surface
            };

            Shadow("card-shadow", surface)
                .elevation(self.elevation)
                .radius(radius)
                .into_element(context)
        })
    }
}

/// Owned view adapter lets Card rebuild its Container composition while the
/// public `View` contract continues to lower children by reference.
struct CardChild<V: View>(Rc<V>);

impl<V: View> View for CardChild<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        self.0.into_element(context)
    }
}

#[cfg(test)]
mod tests {
    use super::{Card, CardView};
    use crate::{modifiers::shadow::ShadowElevation, view::EmptyView};

    #[test]
    fn defaults_to_panel_with_standard_padding_radius_and_elevation() {
        let card = Card("card", EmptyView);

        assert_eq!(card.padding, None);
        assert_eq!(card.radius, None);
        assert_eq!(card.elevation, ShadowElevation::Medium);
        assert!(!card.border);
    }

    #[test]
    fn builders_update_card_options() {
        let card = Card("card", EmptyView)
            .padding(0.0)
            .radius(4.0)
            .elevation(ShadowElevation::Large)
            .border(true);

        assert_eq!(card.padding, Some(0.0));
        assert_eq!(card.radius, Some(4.0));
        assert_eq!(card.elevation, ShadowElevation::Large);
        assert!(card.border);
    }

    #[test]
    #[should_panic(expected = "Card padding must be finite and nonnegative")]
    fn rejects_negative_padding() {
        let _: CardView<_> = Card("card", EmptyView).padding(-1.0);
    }

    #[test]
    #[should_panic(expected = "Card padding must be finite and nonnegative")]
    fn rejects_nonfinite_padding() {
        let _: CardView<_> = Card("card", EmptyView).padding(f32::NAN);
    }

    #[test]
    #[should_panic(expected = "Card radius must be finite and nonnegative")]
    fn rejects_nonfinite_radius() {
        let _: CardView<_> = Card("card", EmptyView).radius(f32::INFINITY);
    }

    #[test]
    #[should_panic(expected = "Card radius must be finite and nonnegative")]
    fn rejects_negative_radius() {
        let _: CardView<_> = Card("card", EmptyView).radius(-1.0);
    }

    #[test]
    #[should_panic(expected = "Card ID must not be empty")]
    fn rejects_empty_id() {
        let _ = Card("", EmptyView);
    }
}
