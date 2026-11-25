/// Whether the opt-in RustUI G5 bundle is enabled on macOS.
#[doc(hidden)]
pub fn fast_g5_profile_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    cfg!(target_os = "macos")
        && *ON.get_or_init(|| std::env::var("RUSTUI_G5_PROFILE").as_deref() == Ok("1"))
}
