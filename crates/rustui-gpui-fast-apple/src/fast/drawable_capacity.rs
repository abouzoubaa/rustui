// RustUI G5-3: experimental two-free-slot acquisition policy; default off.
use std::sync::OnceLock;
pub(crate) fn reserve_two() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("GPUI_FAST_DRAWABLE_RESERVE_TWO").as_deref() == Ok("1"))
}
pub(crate) fn policy(count: usize, pool: usize, two: bool) -> bool {
    if two {
        pool >= 2 && count <= pool - 2
    } else {
        count < pool
    }
}
pub(crate) fn allowed(count: usize, pool: usize) -> bool {
    policy(count, pool, reserve_two())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_slots_must_be_free_before_acquisition() {
        assert!(policy(0, 3, true));
        assert!(policy(1, 3, true));
        assert!(!policy(2, 3, true));
        assert!(!policy(3, 3, true));
        assert!(policy(0, 2, true));
        assert!(!policy(1, 2, true));
        assert!(!policy(0, 1, true));
        assert!(!policy(0, 0, true));
        assert!(policy(2, 3, false));
        assert!(!policy(3, 3, false));
        assert!(!policy(0, 0, false));
    }
    #[test]
    fn reserve_is_off_by_default() {
        assert!(!reserve_two());
    }
}
