//! Native GPUI image loading and fitting.

use gpui::{InteractiveElement as _, ParentElement as _};
use std::path::{Path, PathBuf};

use gpui::{
    AnyElement, IntoElement, SharedString, StatefulInteractiveElement, Styled, StyledImage, div,
    img, px,
};
use rustui_core::Identity;

use crate::{
    components::theme::{required_style_with_artwork, selector, styled_container},
    view::{EmptyView, RenderContext, View},
};

/// An image source understood by the GPUI resource loader.
///
/// String inputs to [`Image`] name embedded application assets. Pass a
/// `PathBuf` or `&Path` to load a filesystem image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageSource {
    /// An embedded asset path resolved by the application's GPUI asset source.
    Asset(SharedString),
    /// A filesystem path loaded by GPUI's native image resource loader.
    File(PathBuf),
}

impl From<&str> for ImageSource {
    fn from(value: &str) -> Self {
        Self::Asset(value.into())
    }
}

impl From<String> for ImageSource {
    fn from(value: String) -> Self {
        Self::Asset(value.into())
    }
}

impl From<SharedString> for ImageSource {
    fn from(value: SharedString) -> Self {
        Self::Asset(value)
    }
}

impl From<PathBuf> for ImageSource {
    fn from(value: PathBuf) -> Self {
        Self::File(value)
    }
}

impl From<&Path> for ImageSource {
    fn from(value: &Path) -> Self {
        Self::File(value.to_path_buf())
    }
}

/// Native GPUI image fitting behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// Scale the complete image inside the view bounds.
    #[default]
    Contain,
    /// Scale the image to cover the view bounds, cropping excess edges.
    Cover,
    /// Stretch the image to exactly fill the view bounds.
    Fill,
    /// Scale down only when the image exceeds the view bounds.
    ScaleDown,
    /// Keep the image at its original size.
    None,
}

impl ImageFit {
    fn native(self) -> gpui::ObjectFit {
        match self {
            Self::Contain => gpui::ObjectFit::Contain,
            Self::Cover => gpui::ObjectFit::Cover,
            Self::Fill => gpui::ObjectFit::Fill,
            Self::ScaleDown => gpui::ObjectFit::ScaleDown,
            Self::None => gpui::ObjectFit::None,
        }
    }
}

/// An image with optional explicit bounds, fit behavior, and accessible name.
#[derive(Clone, Debug)]
pub struct ImageView {
    source: ImageSource,
    size: Option<(f32, f32)>,
    fit: ImageFit,
    utility: bool,
    accessibility_label: Option<SharedString>,
    accessibility_id: Option<Identity>,
}

impl ImageView {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            source: source.into(),
            size: None,
            fit: ImageFit::default(),
            utility: false,
            accessibility_label: None,
            accessibility_id: None,
        }
    }

    /// Sets explicit logical-pixel width and height.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0,
            "image dimensions must be finite and positive"
        );
        self.size = Some((width, height));
        self
    }

    /// Selects the native GPUI image object-fit behavior.
    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    /// Selects the macOS 27 utility-image theme rules, including its 24×24 size.
    pub fn utility(mut self) -> Self {
        self.utility = true;
        self
    }

    /// Gives a meaningful image an accessible name. Unlabeled images remain
    /// unannounced; provide a label for content images and omit it for decoration.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets a stable accessibility element ID.
    ///
    /// Labeled images otherwise derive an ID from their source and label.
    /// Provide distinct IDs when identical labeled images appear as siblings.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.accessibility_id = Some(id.into());
        self
    }
}

impl ImageSource {
    fn native(&self) -> gpui::ImageSource {
        match self {
            Self::Asset(path) => {
                gpui::ImageSource::Resource(gpui::Resource::Embedded(path.clone()))
            }
            Self::File(path) => gpui::ImageSource::from(path.clone()),
        }
    }
}

/// Creates a GPUI-backed image from an embedded asset name or filesystem path.
///
/// String arguments are embedded asset names; use `PathBuf` or `&Path` for a
/// filesystem image. Unspecified dimensions use the decoded image's intrinsic
/// size.
#[allow(non_snake_case)]
pub fn Image(source: impl Into<ImageSource>) -> ImageView {
    ImageView::new(source)
}

impl View for ImageView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut image_selector = selector(context);
        let part = if self.utility {
            image_selector.chrome = "utility".into();
            "images"
        } else {
            "image"
        };
        let style = required_style_with_artwork(context, "image", part, &image_selector);
        let mut image = img(self.source.native()).object_fit(self.fit.native());
        let size = self
            .size
            .or_else(|| Some((*style.lengths.get("width")?, *style.lengths.get("height")?)));
        if let Some((width, height)) = size {
            image = image.w(px(width)).h(px(height));
        }
        // Clip image pixels to the record's corner radius; a rounded container
        // alone leaves the child image square.
        let mut container = styled_container(div(), &style).overflow_hidden();
        if let Some((width, height)) = self.size {
            container = container.w(px(width)).h(px(height));
        }
        match &self.accessibility_label {
            Some(label) => {
                let id = self.accessibility_id.clone().unwrap_or_else(|| {
                    Identity::from(format!("rustui-image:{:?}:{label}", self.source))
                });
                container
                    .id(id.as_str().to_owned())
                    .role(gpui::Role::Image)
                    .aria_label(label.clone())
                    .child(image)
                    .into_any_element()
            }
            None => container.child(image).into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_asset_names_and_paths_to_native_resources() {
        let asset = ImageSource::from("images/logo.png").native();
        assert!(matches!(
            asset,
            gpui::ImageSource::Resource(gpui::Resource::Embedded(path))
                if path.as_ref() == "images/logo.png"
        ));

        let file = ImageSource::from(PathBuf::from("/tmp/photo.png")).native();
        assert!(matches!(
            file,
            gpui::ImageSource::Resource(gpui::Resource::Path(path))
                if path.as_ref() == Path::new("/tmp/photo.png")
        ));
    }

    #[test]
    fn explicit_dimensions_are_retained() {
        let image = Image("images/logo.png").size(32.0, 24.0);
        assert_eq!(image.size, Some((32.0, 24.0)));
    }

    #[test]
    fn utility_role_selects_utility_artwork() {
        let image = Image("images/tool.png").utility();
        assert!(image.utility);
    }

    #[test]
    #[should_panic(expected = "image dimensions must be finite and positive")]
    fn rejects_non_positive_dimensions() {
        Image("images/logo.png").size(32.0, 0.0);
    }

    #[test]
    #[should_panic(expected = "image dimensions must be finite and positive")]
    fn rejects_non_finite_dimensions() {
        Image("images/logo.png").size(f32::INFINITY, 24.0);
    }
}
