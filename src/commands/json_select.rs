//! `json select`: select a value by a simple path like `a.b[0].c`.
//! Supports input from --text, --file, or stdin with optional JSON5 relaxed parsing.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use crate::helpers::style;

/// Input source priority: --file > --text > stdin
fn get_input(text: &Option<String>, file: &Option<PathBuf>) -> Result<String> {
    if let Some(path) = file {
        return fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()));
    }

    if let Some(t) = text {
        return Ok(t.clone());
    }

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Detect common JSON format issues and provide helpful error messages
fn analyze_json_error(input: &str, error: &serde_json::Error) -> String {
    let mut msg = String::from("Invalid JSON format");

    // Check for common issues
    if input.contains("'{") || input.contains("}'") {
        msg.push_str(" - avoid single quotes around the entire JSON");
    } else if input.contains(": '") {
        msg.push_str(" - string values must use double quotes, not single quotes");
    } else if input.chars().any(|c| c.is_alphabetic()) && !input.contains('"') {
        msg.push_str(" - keys must be in double quotes");
    }

    msg.push_str("\n\nExamples of valid JSON:");
    msg.push_str("\n  PowerShell: --text '{\"a\":{\"b\":[1,2,3]}}'");
    msg.push_str("\n  CMD:        --text \"{\\\"a\\\":{\\\"b\\\":[1,2,3]}}\"");

    msg.push_str("\n\nAlternative options:");
    msg.push_str("\n  Use --json5 for relaxed parsing: --json5 --text '{a:{b:[1,2,3]}}'");
    msg.push_str(
        "\n  Use stdin: echo '{\"a\":{\"b\":[1,2,3]}}' | swiftline json select --path a.b[2]",
    );
    msg.push_str("\n  Use file: swiftline json select --file data.json --path a.b[2]");

    msg.push_str(&format!("\n\nOriginal error: {error}"));
    msg
}

/// Parse JSON with fallback to JSON5 if enabled and strict parsing fails
fn parse_json(input: &str, use_json5: bool) -> Result<Value> {
    // Try strict JSON first
    match serde_json::from_str(input) {
        Ok(value) => Ok(value),
        Err(strict_error) => {
            if use_json5 {
                // Try JSON5 fallback
                match json5::from_str(input) {
                    Ok(value) => Ok(value),
                    Err(json5_error) => {
                        anyhow::bail!(
                            "Failed to parse as JSON or JSON5\n\nStrict JSON error: {}\nJSON5 error: {}",
                            strict_error, json5_error
                        );
                    }
                }
            } else {
                // Provide helpful error for strict JSON failure
                anyhow::bail!("{}", analyze_json_error(input, &strict_error));
            }
        }
    }
}

/// Simple path resolver supporting object and array access:
/// - Dots traverse objects: `a.b.c`
/// - [idx] traverses arrays: `items[0]`
/// - One [idx] per segment is supported, e.g. `a.b[2].c`
fn get_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = value;
    for seg in path.split('.') {
        if seg.is_empty() {
            return None;
        }

        // Handle `name[index]` form
        if let Some((name, rest)) = seg.split_once('[') {
            if !name.is_empty() {
                cur = cur.get(name)?;
            }
            if !rest.ends_with(']') {
                return None;
            }
            let idx_str = &rest[..rest.len() - 1];
            let idx: usize = idx_str.parse().ok()?;
            cur = cur.get(idx)?;
        } else {
            cur = cur.get(seg)?;
        }
    }
    Some(cur)
}

/// Select JSON value by path from text input, file, or stdin.
pub fn run(text: Option<String>, file: Option<PathBuf>, json5: bool, path: String) -> Result<()> {
    style::title("JSON Select");

    let raw = get_input(&text, &file)?;
    let json = parse_json(raw.trim(), json5)?;

    match get_by_path(&json, &path) {
        Some(v) => {
            // Pretty JSON; colored if TTY, plain otherwise.
            let pretty = colored_json::to_colored_json_auto(v)?;
            println!("{pretty}");
        }
        None => {
            // Intentionally minimal for scripting pipelines.
            println!("(null)");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_by_path_object_access() {
        let data = json!({"a": {"b": {"c": "value"}}});
        assert_eq!(get_by_path(&data, "a.b.c"), Some(&json!("value")));
    }

    #[test]
    fn test_get_by_path_array_access() {
        let data = json!({"items": [1, 2, 3]});
        assert_eq!(get_by_path(&data, "items[0]"), Some(&json!(1)));
        assert_eq!(get_by_path(&data, "items[2]"), Some(&json!(3)));
    }

    #[test]
    fn test_get_by_path_mixed_access() {
        let data = json!({"a": {"b": [{"c": "found"}]}});
        assert_eq!(get_by_path(&data, "a.b[0].c"), Some(&json!("found")));
    }

    #[test]
    fn test_get_by_path_missing_path() {
        let data = json!({"a": {"b": "value"}});
        assert_eq!(get_by_path(&data, "a.x"), None);
        assert_eq!(get_by_path(&data, "missing"), None);
    }

    #[test]
    fn test_get_by_path_invalid_array_index() {
        let data = json!({"items": [1, 2]});
        assert_eq!(get_by_path(&data, "items[5]"), None);
        assert_eq!(get_by_path(&data, "items[abc]"), None);
    }

    #[test]
    fn test_get_by_path_empty_segments() {
        let data = json!({"a": "value"});
        assert_eq!(get_by_path(&data, ""), None);
        assert_eq!(get_by_path(&data, "a..b"), None);
    }

    #[test]
    fn test_parse_json_strict() {
        let valid = r#"{"a": {"b": [1, 2, 3]}}"#;
        assert!(parse_json(valid, false).is_ok());

        let invalid = r#"{a: {b: [1, 2, 3]}}"#;
        assert!(parse_json(invalid, false).is_err());
    }

    #[test]
    fn test_parse_json_json5() {
        let json5_input = r#"{a: {b: [1, 2, 3]}}"#;
        assert!(parse_json(json5_input, true).is_ok());
        assert!(parse_json(json5_input, false).is_err());
    }

    #[test]
    fn test_analyze_json_error_contains_helpful_text() {
        let input = r#"{a: {b: [1, 2, 3]}}"#;
        let error = serde_json::from_str::<Value>(input).unwrap_err();
        let msg = analyze_json_error(input, &error);

        assert!(msg.contains("keys must be in double quotes"));
        assert!(msg.contains("--json5"));
        assert!(msg.contains("PowerShell"));
        assert!(msg.contains("CMD"));
    }
}
