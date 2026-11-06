//! Standalone Text component specimen composed through the public RustUI API.

use rustui::prelude::*;

const TEXT_SPECIMEN_SAMPLES: [(&str, &str); 3] = [
    ("Heading", "Keep the message clear"),
    (
        "Body",
        "Use concise supporting copy to explain the information that follows.",
    ),
    (
        "Wrapping",
        "Text keeps its reading order as a longer sentence wraps within the available content width.",
    ),
];

pub fn demo() -> impl View {
    Container(
        VStack::new()
            .spacing(8.0)
            .children(
                TEXT_SPECIMEN_SAMPLES
                    .iter()
                    .enumerate()
                    .map(|(index, (role, sample))| {
                        let specimen = if index == 0 {
                            Text(*sample)
                                .font_size(22.0)
                                .font_weight(TextWeight::Semibold)
                        } else {
                            Text(*sample)
                        };
                        VStack::new()
                            .spacing(2.0)
                            .child(Text(*role).font_size(11.0))
                            .child(specimen)
                    }),
            ),
    )
    .width(420.0)
    .padding(12.0)
    .border()
}

#[cfg(test)]
mod tests {
    use super::TEXT_SPECIMEN_SAMPLES;

    #[test]
    fn text_specimen_covers_heading_body_and_wrapping_samples() {
        assert_eq!(
            TEXT_SPECIMEN_SAMPLES,
            [
                ("Heading", "Keep the message clear"),
                (
                    "Body",
                    "Use concise supporting copy to explain the information that follows."
                ),
                (
                    "Wrapping",
                    "Text keeps its reading order as a longer sentence wraps within the available content width."
                ),
            ]
        );
    }
}
