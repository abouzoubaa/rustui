//! Bounded recent event history owned by the Playground story host.

use std::collections::VecDeque;

use rustui::{
    localization::MessageArguments,
    prelude::{LocalizedText, VStack, View},
};

const RECENT_EVENT_LIMIT: usize = 5;

struct EventEntry {
    message: &'static str,
    arguments: MessageArguments,
}

#[derive(Default)]
pub struct EventLog {
    entries: VecDeque<EventEntry>,
    total_count: usize,
}

impl EventLog {
    pub fn record(&mut self, message: &'static str, arguments: MessageArguments) {
        self.entries.push_back(EventEntry { message, arguments });
        if self.entries.len() > RECENT_EVENT_LIMIT {
            self.entries.pop_front();
        }
        self.total_count = self.total_count.saturating_add(1);
    }

    pub fn record_simple(&mut self, message: &'static str) {
        self.record(message, MessageArguments::new());
    }

    pub fn view(&self) -> impl View {
        let mut content = VStack::new().spacing(4.0);
        if self.entries.is_empty() {
            return content.child(LocalizedText("lab-events-empty"));
        }
        content =
            content
                .child(LocalizedText("lab-events-total").with_arguments(
                    MessageArguments::new().with("count", self.total_count as i64),
                ));
        for entry in &self.entries {
            content =
                content.child(LocalizedText(entry.message).with_arguments(entry.arguments.clone()));
        }
        content
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{EventLog, RECENT_EVENT_LIMIT};

    #[test]
    fn recent_events_keep_order_and_total_after_truncation() {
        let mut log = EventLog::default();
        for event in ["one", "two", "three", "four", "five", "six"] {
            log.record_simple(event);
        }
        assert_eq!(
            log.entries
                .iter()
                .map(|entry| entry.message)
                .collect::<Vec<_>>(),
            ["two", "three", "four", "five", "six"]
        );
        assert_eq!(log.entries.len(), RECENT_EVENT_LIMIT);
        assert_eq!(log.total_count, 6);
    }

    #[test]
    fn clear_removes_events_and_resets_total() {
        let mut log = EventLog::default();
        log.record_simple("changed state");
        log.clear();
        assert!(log.entries.is_empty());
        assert_eq!(log.total_count, 0);
    }
}
