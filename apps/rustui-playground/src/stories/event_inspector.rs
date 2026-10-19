//! In-memory event history owned by the Playground story host component.

#[derive(Default)]
pub struct EventLog {
    entries: Vec<String>,
}

impl EventLog {
    pub fn record(&mut self, event: impl Into<String>) {
        self.entries.push(event.into());
    }

    pub fn summary(&self) -> String {
        self.entries
            .last()
            .map(|event| format!("Last event: {event} · {} total", self.entries.len()))
            .unwrap_or_else(|| "No events recorded".into())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
