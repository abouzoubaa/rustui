//! Illustrative active and inactive window-control composition for the Playground.

use rustui::prelude::*;

const TRAFFIC_LIGHTS: [(&str, &str, &str); 3] = [
    ("close", "Close", "🔴"),
    ("minimize", "Minimize", "🟡"),
    ("zoom", "Zoom", "🟢"),
];

pub fn demo() -> impl View {
    Component("window-controls-gallery", WindowControlsGallery::default)
}

#[component]
struct WindowControlsGallery {
    #[state(true)]
    active: bool,
    #[state(String::from("None"))]
    last_control: String,
}

impl WindowControlsGallery {
    fn select_close(&mut self) {
        self.active = true;
        self.last_control = String::from("Close");
    }

    fn select_minimize(&mut self) {
        self.active = true;
        self.last_control = String::from("Minimize");
    }

    fn select_zoom(&mut self) {
        self.active = true;
        self.last_control = String::from("Zoom");
    }

    fn last_control_feedback(&self) -> String {
        format!("Last selected control: {}", self.last_control())
    }

    fn simulate_inactive(&mut self) {
        self.active = false;
    }

    fn light_label(name: &str, color: &str, active: bool) -> String {
        let state = if active { "active" } else { "inactive" };
        let marker = if active { color } else { "⚪" };
        format!("{marker} {name} · {state}")
    }

    fn light_accessibility_label(name: &str, active: bool) -> String {
        let state = if active { "active" } else { "inactive" };
        format!("Illustrative {name} light; preview window {state}")
    }
}

impl View for WindowControlsGallery {
    fn body(&self) -> impl View {
        let active = self.active();
        let lights = TRAFFIC_LIGHTS.iter().map(|(id, name, color)| {
            let action = match *id {
                "close" => Self::select_close,
                "minimize" => Self::select_minimize,
                "zoom" => Self::select_zoom,
                _ => unreachable!("traffic-light ID is declared in TRAFFIC_LIGHTS"),
            };
            Button(
                Self::light_label(name, color, active),
                self.action(action),
            )
            .id(format!("window-control-{id}"))
            .button_style(ButtonStyle::Plain)
            .toggled(active)
            .accessibility_label(Self::light_accessibility_label(name, active))
        });

        VStack::new()
            .spacing(8.0)
            .child(Text("Illustrative window controls"))
            .child(HStack::new().spacing(8.0).children(lights))
            .child(
                Button(
                    "Simulate inactive window",
                    self.action(Self::simulate_inactive),
                )
                .id("window-controls-inactive")
                .toggled(!active),
            )
            .child(Text(
                "Illustration only · clicks change preview state, not system controls; no native chrome",
            ))
            .child(Text(self.last_control_feedback()))
    }
}

#[cfg(test)]
mod tests {
    use super::{TRAFFIC_LIGHTS, WindowControlsGallery};

    #[test]
    fn window_control_buttons_reflect_owner_active_state() {
        assert_eq!(
            TRAFFIC_LIGHTS,
            [
                ("close", "Close", "🔴"),
                ("minimize", "Minimize", "🟡"),
                ("zoom", "Zoom", "🟢"),
            ]
        );

        let mut gallery = WindowControlsGallery::default();
        assert!(gallery.active());
        for (_, name, color) in TRAFFIC_LIGHTS {
            assert_eq!(
                WindowControlsGallery::light_label(name, color, gallery.active()),
                format!("{color} {name} · active")
            );
            assert_eq!(
                WindowControlsGallery::light_accessibility_label(name, gallery.active()),
                format!("Illustrative {name} light; preview window active")
            );
        }

        gallery.simulate_inactive();
        assert!(!gallery.active());
        for (_, name, color) in TRAFFIC_LIGHTS {
            assert_eq!(
                WindowControlsGallery::light_label(name, color, gallery.active()),
                format!("⚪ {name} · inactive")
            );
        }

        gallery.simulate_inactive();
        gallery.select_close();
        assert!(gallery.active());
        assert_eq!(gallery.last_control(), "Close");
        assert_eq!(gallery.last_control_feedback(), "Last selected control: Close");
        gallery.simulate_inactive();
        gallery.select_minimize();
        assert!(gallery.active());
        assert_eq!(gallery.last_control(), "Minimize");
        assert_eq!(
            gallery.last_control_feedback(),
            "Last selected control: Minimize"
        );
        gallery.simulate_inactive();
        gallery.select_zoom();
        assert!(gallery.active());
        assert_eq!(gallery.last_control(), "Zoom");
        assert_eq!(gallery.last_control_feedback(), "Last selected control: Zoom");
    }
}
