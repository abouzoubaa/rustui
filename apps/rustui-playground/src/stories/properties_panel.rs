//! Typed editable properties used by the current Button story.

use rustui::prelude::ButtonStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonProperties {
    pub style: ButtonStyle,
    pub disabled: bool,
}

impl ButtonProperties {
    pub fn cycle_style(&mut self) {
        self.style = match self.style {
            ButtonStyle::Automatic => ButtonStyle::Plain,
            ButtonStyle::Plain => ButtonStyle::Bordered,
            ButtonStyle::Bordered => ButtonStyle::BorderedProminent,
            ButtonStyle::BorderedProminent => ButtonStyle::Borderless,
            ButtonStyle::Borderless => ButtonStyle::Automatic,
        };
    }
}

impl Default for ButtonProperties {
    fn default() -> Self {
        Self {
            style: ButtonStyle::Automatic,
            disabled: false,
        }
    }
}
