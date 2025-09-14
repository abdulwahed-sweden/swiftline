//! Entry point: enable colors, set up logging, parse CLI, dispatch subcommands.

use anyhow::Result;
use clap::{CommandFactory, Parser};
use env_logger::Env;
use log::debug;

mod cli;
mod commands;
mod helpers;

use cli::{Cli, Commands, HttpCommands, JsonCommands};

#[tokio::main]
async fn main() -> Result<()> {
    // Enable ANSI colors on Windows and set small style helpers.
    helpers::style::init_colors();

    // Parse CLI flags/subcommands.
    let cli = Cli::parse();

    // Configure logger based on -v / -vv. Defaults to "warn".
    let default_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(default_level)).init();

    debug!("CLI args: {cli:?}");

    match cli.command {
        // No subcommand: print help (exit code 0).
        None => {
            let mut cmd = Cli::command();
            cmd.print_help().ok();
            println!();
            Ok(())
        }

        // http get <...>
        Some(Commands::Http(HttpCommands::Get {
            url,
            headers,
            timeout,
            save,
            pretty,
        })) => commands::http_get::run(&url, &headers, timeout, save, pretty).await,

        // json select --path <...> [--text <...>] [--file <...>] [--json5]
        Some(Commands::Json(JsonCommands::Select {
            text,
            file,
            json5,
            path,
        })) => commands::json_select::run(text, file, json5, path),
    }
}
