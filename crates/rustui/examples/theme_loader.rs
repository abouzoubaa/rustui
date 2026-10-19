use std::{env, io};

use rustui::theme::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "themes/macos27.json".to_owned());
    let theme = load_theme(path)?;
    let profile = theme
        .select_profile("macos", 27)
        .ok_or_else(|| io::Error::other("theme has no matching macOS profile"))?;
    let colors = profile.colors(None);

    println!(
        "Loaded {} profile `{}`; default {:?} appearance uses window background {}",
        theme.framework.name, profile.id, profile.appearance.default, colors.window_background
    );
    Ok(())
}
