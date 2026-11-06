//! General decoration and typed child identity through public RustUI APIs.

use rustui::{
    component,
    prelude::*,
    theme::{AppearanceMode, activate_theme},
};

const SHARED_CHILD_ID: &str = "decoration-shared-input";

pub fn demo() -> impl View {
    Component("decoration-gallery", DecorationGallery::default)
}

#[component]
struct DecorationGallery {
    #[state(false)]
    emphasized: bool,
    #[state(true)]
    show_a: bool,
    #[state(0)]
    parent_updates: usize,
    #[state(0)]
    button_hits: usize,
    #[state(String::from("Background A"))]
    background_a: String,
    #[state(String::from("Background B"))]
    background_b: String,
    #[state(String::from("Border A"))]
    border_a: String,
    #[state(String::from("Border B"))]
    border_b: String,
}

impl DecorationGallery {
    fn toggle_style(&mut self) {
        self.emphasized = !self.emphasized;
    }
    fn toggle_a(&mut self) {
        self.show_a = !self.show_a;
    }
    fn update_parent(&mut self) {
        self.parent_updates += 1;
    }
    fn hit(&mut self) {
        self.button_hits += 1;
    }
}

fn appearance(mode: AppearanceMode) -> AppAction {
    AppAction::new(move |cx| {
        let result = crate::bundled_theme()
            .map_err(str::to_owned)
            .and_then(|theme| {
                activate_theme(&theme, Some(mode), cx).map_err(|error| error.to_string())
            });
        if let Err(error) = result {
            eprintln!("Decoration gallery appearance failed: {error}");
        }
    })
}

impl View for DecorationGallery {
    fn body(&self) -> impl View {
        let surface = if self.emphasized() {
            Surface::Sidebar
        } else {
            Surface::Panel
        };
        let radius = if self.emphasized() { 12.0 } else { 0.0 };
        let width = if self.emphasized() { 3.0 } else { 1.0 };
        let color = if self.emphasized() {
            BorderColor::Accent
        } else {
            BorderColor::Border
        };
        // Containers establish geometry only; no additional keyed/component
        // owner hides the decoration wrapper's child namespace responsibility.
        let background_a = if self.show_a() {
            AnyView::new(
                Background(
                    "deco-a",
                    Container(
                        TextField("Background A", self.background_a_binding()).id(SHARED_CHILD_ID),
                    )
                    .width(240.0)
                    .padding(8.0),
                    surface,
                )
                .radius(radius),
            )
        } else {
            AnyView::new(Text("Background A omitted"))
        };
        let border_a = if self.show_a() {
            AnyView::new(
                Border(
                    "deco-a",
                    Container(TextField("Border A", self.border_a_binding()).id(SHARED_CHILD_ID))
                        .width(240.0)
                        .padding(8.0),
                )
                .width(width)
                .radius(radius)
                .color(color),
            )
        } else {
            AnyView::new(Text("Border A omitted"))
        };
        VStack::new().spacing(8.0)
            .child(Text("Background and Border modifiers"))
            .child(super::locale_selector::LocaleSelector::new())
            .child(HStack::new().spacing(6.0)
                .child(Button("Toggle decoration", self.action(Self::toggle_style)).id("decoration-style"))
                .child(Button("Update parent", self.action(Self::update_parent)).id("decoration-parent"))
                .child(Button("Omit / restore A", self.action(Self::toggle_a)).id("decoration-mount")))
            .child(HStack::new().spacing(6.0)
                .child(Button("Light appearance", appearance(AppearanceMode::Light)).id("decoration-light"))
                .child(Button("Dark appearance", appearance(AppearanceMode::Dark)).id("decoration-dark")))
            .child(Text(format!("Parent updates {} · button hits {} · emphasized {}", self.parent_updates(), self.button_hits(), self.emphasized())))
            .child(Background("translated-text", Container(LocalizedText("rustui-action-close"))
                .padding_edges(LogicalInsets::new(8.0, 8.0, 20.0, 6.0).expect("fixed insets are valid")), surface).radius(radius))
            .child(Border("action-button", Container(Button("Decorated owner action", self.action(Self::hit)).id("decoration-action"))
                .padding(8.0)).width(width).radius(radius).color(color))
            .child(Text("Two Background siblings, shared explicit child ID"))
            .child(HStack::new().spacing(8.0)
                .child(background_a)
                .child(Background("deco-b", Container(TextField("Background B", self.background_b_binding()).id(SHARED_CHILD_ID))
                    .width(240.0).padding(8.0), Surface::Background).radius(radius)))
            .child(Text(format!("Background mirrors: A [{}] · B [{}]", self.background_a(), self.background_b())))
            .child(Text("Two Border siblings, reusing wrapper A/B IDs across families"))
            .child(HStack::new().spacing(8.0)
                .child(border_a)
                .child(Border("deco-b", Container(TextField("Border B", self.border_b_binding()).id(SHARED_CHILD_ID))
                    .width(240.0).padding(8.0)).width(2.0).radius(radius).color(BorderColor::FocusRing)))
            .child(Text(format!("Border mirrors: A [{}] · B [{}]", self.border_a(), self.border_b())))
            .child(Border("composed", Background("composed", Container(Text("Composable fill and stroke"))
                .padding(10.0), surface).radius(radius)).width(width).radius(radius).color(color))
            .child(Text("Edit distinct values, toggle style/theme/locale and update parent: all mirrors must stay independent. Omit/restore A preserves root strings. Rounding does not clip children. FocusRing is a decorative color token, not focus state."))
    }
}
