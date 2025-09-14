//! Spinner helper: a minimal, readable spinner for async tasks.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner with a simple, readable template.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .expect("valid spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(msg.to_string());
    pb
}
