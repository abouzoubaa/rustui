//! Bounded recent event history owned by the Playground story host component.

use std::collections::VecDeque;

const RECENT_EVENT_LIMIT: usize = 5;

#[derive(Default)]
pub struct EventLog {
    entries: VecDeque<String>,
    total_count: usize,
}

impl EventLog {
    pub fn record(&mut self, event: impl Into<String>) {
        self.entries.push_back(event.into());
        if self.entries.len() > RECENT_EVENT_LIMIT {
            self.entries.pop_front();
        }
        self.total_count = self.total_count.saturating_add(1);
    }

    pub fn summary(&self) -> String {
        if self.entries.is_empty() {
            return String::from("No events recorded");
        }

        let recent = self
            .entries
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" → ");
        format!("Recent events: {recent} · {} total", self.total_count)
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
    fn summary_shows_recent_events_in_order_and_reports_total() {
        let mut log = EventLog::default();
        for event in ["one", "two", "three", "four", "five", "six"] {
            log.record(event);
        }

        assert_eq!(
            log.summary(),
            "Recent events: two → three → four → five → six · 6 total"
        );
        assert_eq!(log.entries.len(), RECENT_EVENT_LIMIT);
        assert_eq!(log.total_count, 6);
    }

    #[test]
    fn clear_removes_events_from_summary() {
        let mut log = EventLog::default();
        log.record("changed state");
        log.clear();

        assert_eq!(log.summary(), "No events recorded");
        assert!(log.entries.is_empty());
        assert_eq!(log.total_count, 0);
    }
}
