//! CLI layout: arguments, options, and subcommands.

use clap::{ArgAction, Parser, Subcommand};

/// Swiftline — minimal, fast CLI with only the essentials.
#[derive(Parser, Debug)]
#[command(
    name = "swiftline",
    version,
    about = "Minimal, fast CLI with just what matters"
)]
pub struct Cli {
    /// Increase verbosity (-v -> info, -vv -> debug)
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Optional subcommand; prints help if omitted
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// HTTP utilities
    #[command(subcommand)]
    Http(HttpCommands),

    /// JSON utilities
    #[command(subcommand)]
    Json(JsonCommands),
}

#[derive(Subcommand, Debug)]
pub enum HttpCommands {
    /// GET a URL (headers -H, timeout, optional save, pretty JSON)
    Get {
        /// URL to GET
        url: String,

        /// Repeatable header key:value, e.g. -H "Accept: application/json"
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,

        /// Timeout in seconds (default 30)
        #[arg(long)]
        timeout: Option<u64>,

        /// Save response body to this file path (streamed with progress)
        #[arg(long)]
        save: Option<std::path::PathBuf>,

        /// Pretty-print JSON responses (auto-colored)
        #[arg(long)]
        pretty: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum JsonCommands {
    /// Select a value from JSON by a simple path like: data.items[0].name
    Select {
        /// The JSON input; if omitted, reads from stdin
        #[arg(long)]
        text: Option<String>,

        /// Read JSON from file instead of --text or stdin
        #[arg(long)]
        file: Option<std::path::PathBuf>,

        /// Enable relaxed JSON5 parsing (unquoted keys, trailing commas, etc.)
        #[arg(long)]
        json5: bool,

        /// Path like: a.b[0].c  (dot for objects, [index] for arrays)
        #[arg(long)]
        path: String,
    },
}
