//! Typed property inspector composition and stable row identity examples.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Container(Component(
        "property-inspector-gallery",
        PropertyInspectorGallery::default,
    ))
    .width(620.0)
    .padding(12.0)
    .border()
}

#[component]
struct PropertyInspectorGallery {
    #[state("Mira".to_owned())]
    name: String,
    #[state(true)]
    enabled: bool,
    #[state(3)]
    count: i32,
    #[state(false)]
    reordered: bool,
    #[state(false)]
    inspector_disabled: bool,
    #[state(false)]
    count_disabled: bool,
    #[state(String::from("Sibling value"))]
    sibling_name: String,
}

impl PropertyInspectorGallery {
    fn toggle_order(&mut self) {
        self.reordered = !self.reordered;
    }

    fn toggle_inspector_disabled(&mut self) {
        self.inspector_disabled = !self.inspector_disabled;
    }

    fn toggle_count_disabled(&mut self) {
        self.count_disabled = !self.count_disabled;
    }
}

impl View for PropertyInspectorGallery {
    fn body(&self) -> impl View {
        let mut rows = vec![
            PropertyInspectorRow::text("name", "Name", self.name_binding()),
            PropertyInspectorRow::toggle("enabled", "Enabled", self.enabled_binding()),
            PropertyInspectorRow::integer("count", "Count", self.count_binding())
                .disabled(self.count_disabled()),
        ];
        if self.reordered {
            rows.swap(0, 2);
        }

        let main_inspector = PropertyInspector("gallery-inspector", rows)
            .expect("gallery IDs are valid")
            .disabled(self.inspector_disabled());
        let sibling_inspector = PropertyInspector(
            "gallery-sibling-inspector",
            [PropertyInspectorRow::text(
                "name",
                "Sibling name",
                self.sibling_name_binding(),
            )],
        )
        .expect("sibling gallery IDs are valid");

        VStack::new()
            .spacing(10.0)
            .child(Text("Typed property inspector").font_weight(TextWeight::Semibold))
            .child(Text(
                "Text, toggle, and integer values write through their owning component bindings.",
            ))
            .child(main_inspector)
            .child(Text(
                "Sibling inspector reuses row ID `name` with a separate owner binding.",
            ))
            .child(sibling_inspector)
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Reorder rows", self.action(Self::toggle_order))
                            .id("property-inspector-reorder"),
                    )
                    .child(
                        Button(
                            "Toggle all disabled",
                            self.action(Self::toggle_inspector_disabled),
                        )
                        .id("property-inspector-disable-all"),
                    )
                    .child(
                        Button(
                            "Toggle Count disabled",
                            self.action(Self::toggle_count_disabled),
                        )
                        .id("property-inspector-disable-count"),
                    ),
            )
            .child(Text(format!(
                "Current values: name={}, enabled={}, count={}, sibling name={}",
                self.name, self.enabled, self.count, self.sibling_name
            )))
            .child(Text(format!(
                "Disabled states: whole inspector={}, Count row={}",
                self.inspector_disabled, self.count_disabled
            )))
    }
}
