use assert_cmd::Command;
use std::fs;

#[test]
fn test_json_select_integration() {
    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd
        .args(&[
            "json",
            "select",
            "--text",
            r#"{"a":{"b":[1,2,3]}}"#,
            "--path",
            "a.b[2]",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"));
}

#[test]
fn test_json5_relaxed_parsing() {
    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd
        .args(&[
            "json",
            "select",
            "--json5",
            "--text",
            r#"{a: {b: [1, 2, 3]}}"#,
            "--path",
            "a.b[2]",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"));
}

#[test]
fn test_file_input() {
    // Create a temporary test file
    let test_file = "test_data.json";
    fs::write(test_file, r#"{"data": {"items": [{"name": "test"}]}}"#).unwrap();

    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd
        .args(&[
            "json",
            "select",
            "--file",
            test_file,
            "--path",
            "data.items[0].name",
        ])
        .output()
        .unwrap();

    // Clean up
    fs::remove_file(test_file).unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"));
}

#[test]
fn test_enhanced_error_message() {
    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd
        .args(&[
            "json",
            "select",
            "--text",
            r#"{a: {b: [1, 2, 3]}}"#,
            "--path",
            "a.b[2]",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid JSON format"));
    assert!(stderr.contains("keys must be in double quotes"));
    assert!(stderr.contains("PowerShell"));
    assert!(stderr.contains("--json5"));
    assert!(stderr.contains("--file"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd.args(&["--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Minimal, fast CLI"));
    assert!(stdout.contains("http"));
    assert!(stdout.contains("json"));
}

#[test]
fn test_json_help_shows_new_flags() {
    let mut cmd = Command::cargo_bin("swiftline").unwrap();
    let output = cmd.args(&["json", "select", "--help"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json5"));
    assert!(stdout.contains("--file"));
}
