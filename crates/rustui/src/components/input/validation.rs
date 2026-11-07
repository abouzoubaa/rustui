//! Shared validation bookkeeping for the single-line input controls.
use std::rc::Rc;

pub(super) type Validator = Rc<dyn Fn(&str) -> Result<(), String>>;

#[derive(Default)]
pub(super) struct ValidationState {
    last: Option<(String, Result<(), String>)>,
    had_binding: bool,
}

impl ValidationState {
    pub(super) fn update(
        &mut self,
        value: &str,
        result: Option<&Result<(), String>>,
        bound_valid: Option<bool>,
    ) -> bool {
        let Some(result) = result else {
            self.last = None;
            self.had_binding = false;
            return false;
        };
        let changed = self
            .last
            .as_ref()
            .is_none_or(|(old_value, old_result)| old_value != value || old_result != result);
        let needs_write = bound_valid
            .is_some_and(|current| changed || !self.had_binding || current != result.is_ok());
        if changed {
            self.last = Some((value.to_owned(), result.clone()));
        }
        self.had_binding = bound_valid.is_some();
        needs_write
    }
}
