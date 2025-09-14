//! `http get`: GET with headers, timeout, optional save with progress,
//! and pretty colored JSON output.

use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use owo_colors::OwoColorize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use crate::helpers::{spinner::spinner, style};

/// Convert repeated "key:value" list into a HeaderMap.
/// Supports multiple values for same key via append.
fn parse_headers(items: &[String]) -> Result<HeaderMap> {
    let mut map: HeaderMap = HeaderMap::new();
    for h in items {
        let (k, v) = h
            .split_once(':')
            .with_context(|| format!("Header must be key:value, got: {h}"))?;

        let key: HeaderName = k
            .trim()
            .parse()
            .with_context(|| format!("Invalid header key: {k}"))?;

        let val: HeaderValue = v
            .trim()
            .parse()
            .with_context(|| format!("Invalid header value for {k}"))?;

        map.append(key, val); // Use append to allow multiple values for same key
    }
    Ok(map)
}

/// Build a progress bar for file downloads when content length is known.
fn sized_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})").unwrap(),
    );
    bar
}

/// Execute HTTP GET request with headers, timeout, optional save, and pretty JSON.
pub async fn run(
    url: &str,
    headers: &[String],
    timeout_secs: Option<u64>,
    save: Option<std::path::PathBuf>,
    pretty: bool,
) -> Result<()> {
    let parsed = Url::parse(url).with_context(|| format!("Invalid URL: {url}"))?;
    let hdrs = parse_headers(headers)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs.unwrap_or(30)))
        .build()?;

    info!("GET {parsed}");

    let pb = spinner("Requesting...");
    let resp = client
        .get(parsed)
        .headers(hdrs)
        .send()
        .await
        .context("Network error while sending request")?;
    let status = resp.status();

    // If saving to file, stream bytes with a progress indicator.
    if let Some(path) = save {
        let total = resp.content_length();
        let mut file = File::create(&path)
            .await
            .with_context(|| format!("Cannot create file: {}", path.display()))?;

        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();

        let pbar = match total {
            Some(t) => sized_bar(t),
            None => spinner("Downloading..."),
        };

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading response stream")?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total.is_some() {
                pbar.set_position(downloaded);
            }
        }

        pbar.finish_and_clear();
        pb.finish_and_clear();

        println!("{} {}", "Status:".bold(), status.to_string().green().bold());
        style::ok(&format!("Saved to: {}", path.display()));
        return Ok(());
    }

    // Not saving: pretty-print JSON or print plain text.
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if pretty && content_type.contains("application/json") {
        let body: Value = resp
            .json()
            .await
            .with_context(|| format!("Failed to parse JSON (status {status})"))?;
        pb.finish_and_clear();

        println!("{} {}", "Status:".bold(), status.to_string().green().bold());

        // Auto-colored JSON (disables colors when not a TTY).
        let pretty_colored = colored_json::to_colored_json_auto(&body)?;
        println!("{pretty_colored}");
    } else {
        let text = resp.text().await?;
        pb.finish_and_clear();

        println!("{} {}", "Status:".bold(), status.to_string().green().bold());
        println!("{text}");
    }

    Ok(())
}
