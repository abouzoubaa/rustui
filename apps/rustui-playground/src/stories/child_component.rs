//! A child-host demo whose state belongs to the retained child component.

use rustui::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DemoDisplayMode {
    Compact,
    Detailed,
}

impl DemoDisplayMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Compact => Self::Detailed,
            Self::Detailed => Self::Compact,
        };
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Detailed => "detailed",
        }
    }
}

#[component]
pub struct ChildComponent {
    label: String,
    #[state(0)]
    count: i32,
    #[state(String::new())]
    text: String,
    #[state(0)]
    workload_increment: u8,
}

#[derive(Clone, PartialEq)]
pub struct ChildProps {
    pub title: String,
    pub workload_increment: u8,
}

impl View for ChildComponent {
    fn body(&self) -> impl View {
        VStack::new()
            .child(Text(self.label.clone()))
            .child(EnvironmentReader(|mode: Option<&DemoDisplayMode>| {
                Text(format!(
                    "RustUI environment display mode: {}",
                    mode.map_or("default (no scoped value)", |mode| mode.label())
                ))
            }))
            .child(Text(format!("Child count: {}", self.count())))
            .child(
                Button("Increment child", self.action(Self::increment))
                    .id("child-increment")
                    .accessibility_label(format!(
                        "Increment {}: count {}",
                        self.label,
                        self.count()
                    )),
            )
            .child(
                TextEditor("Child-owned notes", self.text_binding())
                    .id("child-notes")
                    .height(80.0),
            )
            .spacing(8.0)
    }
}

impl ChildComponent {
    pub fn apply_props(&mut self, props: &ChildProps) {
        self.label.clone_from(&props.title);
        if props.workload_increment > self.workload_increment {
            self.count += i32::from(props.workload_increment - self.workload_increment);
            self.workload_increment = props.workload_increment;
        }
    }

    pub fn named(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}
