//! Tiny styling helpers: enable ANSI on Windows and provide a few colored lines.

use atty::Stream;
use nu_ansi_term; // used just to enable ANSI on Windows
use owo_colors::OwoColorize;

/// Enable ANSI color support on Windows terminals (no-op elsewhere).
pub fn init_colors() {
    #[cfg(windows)]
    let _ = nu_ansi_term::enable_ansi_support();
}

/// Check if stdout is a TTY (used by colored_json to auto-disable colors).
#[inline]
#[allow(dead_code)] // Available for future use
pub fn is_tty() -> bool {
    atty::is(Stream::Stdout)
}

/// Print a bold, underlined title. Keep it short and readable.
pub fn title(msg: &str) {
    println!("{}", msg.bold().underline());
}

/// Print a green success line.
pub fn ok(msg: &str) {
    println!("{}", msg.green().bold());
}

/// Print a yellow warning line.
#[allow(dead_code)] // Available for future use
pub fn warn_line(msg: &str) {
    println!("{}", msg.yellow().bold());
}

/// Print a red error line to stderr.
#[allow(dead_code)] // Available for future use
pub fn err_line(msg: &str) {
    eprintln!("{}", msg.red().bold());
}
