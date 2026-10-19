//! Invalidation decisions consumed by GPUI's notification system.

/// Whether a component mutation requires GPUI to render the owner again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Invalidation {
    #[default]
    None,
    Notify,
}

impl Invalidation {
    pub fn should_notify(self) -> bool {
        matches!(self, Self::Notify)
    }
}

impl From<bool> for Invalidation {
    fn from(changed: bool) -> Self {
        if changed { Self::Notify } else { Self::None }
    }
}
