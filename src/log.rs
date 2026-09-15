//! The mod draws nothing on screen, so `cargo skyline listen` is the only way to see what ran
use std::sync::OnceLock;
use std::time::Instant;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";

static START: OnceLock<Instant> = OnceLock::new();

fn emit(color: &str, tag: &str, args: std::fmt::Arguments<'_>) {
    let ms = START.get_or_init(Instant::now).elapsed().as_millis();
    println!(
        "{}[+{:>4}.{:03}s]{} {}{}[vsync:{}]{} {}",
        DIM,
        ms / 1000,
        ms % 1000,
        RESET,
        BOLD,
        color,
        tag,
        RESET,
        args
    );
}

pub fn startup() {
    emit(
        CYAN,
        "boot",
        format_args!("ssbu-vsync-disabler v{}", env!("CARGO_PKG_VERSION")),
    );
}

pub fn found(name: &str, addr: usize) {
    emit(CYAN, "sig", format_args!("{name} @ {addr:#x}"));
}
