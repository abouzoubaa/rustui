//! Typography samples composed through the public RustUI `Text` API.

use super::primitive_gallery::TEXT_STYLE_EXAMPLES;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Container(
        VStack::new()
            .spacing(12.0)
            .child(
                Text("Text styles")
                    .font_size(20.0)
                    .font_weight(TextWeight::Bold),
            )
            .child(Text(
                "Reference samples using public font size and weight controls.",
            ))
            .children(TEXT_STYLE_EXAMPLES.iter().map(|example| {
                VStack::new()
                    .spacing(2.0)
                    .child(Text(format!("{} · {} pt", example.name, example.size)).font_size(11.0))
                    .child(
                        Text("The quick brown fox jumps over the lazy dog.")
                            .font_size(example.size)
                            .font_weight(example.weight),
                    )
            })),
    )
    .padding(8.0)
    .border()
}
