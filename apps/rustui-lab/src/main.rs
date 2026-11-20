//! Stage G benchmark scenes and compatibility launcher for the Playground.

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--g0-scene") {
        if let Err(error) = rustui_lab::g0::run_cli(&args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    } else {
        rustui_playground::run();
    }
}
