# Swiftline JSON Select Windows Usability Audit

**Date**: 2025-09-14  
**Issue**: Windows users face usability problems with `json select --text` due to shell quoting and strict JSON parsing.

## Root Causes Identified

### 1. Strict JSON Parsing
- `serde_json` requires strict JSON format (keys in double quotes, no trailing commas, etc.)
- Common user input like `{a:{b:[1,2,3]}}` fails with "key must be a string at line 1 column 2"
- Users expect more permissive parsing similar to JavaScript object literals

### 2. Windows Shell Quoting Complexity
- **PowerShell**: Single quotes work differently than Unix shells
- **CMD**: Requires double quote escaping with `""`
- **Git Bash**: Works like Unix but users may not know to use it
- Users get frustrated with quote escaping and give up

### 3. Poor Error Messages
- Current error: "Failed to parse input as JSON" with cryptic serde_json messages
- No guidance on proper JSON format or shell-specific quoting
- No suggestions for alternatives (stdin, file input)

## Reproduction Results

| Input | Shell Behavior | Result |
|-------|---------------|---------|
| `{a:{b:[1,2,3]}}` | Parsed as multiple args | CLI argument error |
| `"{a:{b:[1,2,3]}}"` | Single arg, unquoted keys | "key must be a string" |
| `'{"a":{"b":[1,2,3]}}'` | Works in some shells | Shell-dependent |
| `"{\"a\":{\"b\":[1,2,3]}}"` | CMD-style escaping | Works but complex |

## Implemented Solutions

### A. Enhanced Error Messages
- Detect common JSON format issues
- Provide shell-specific examples
- Suggest alternative input methods

### B. JSON5 Support (`--json5` flag)
- Optional relaxed parsing for unquoted keys, trailing commas, etc.
- Fallback: try strict JSON first, then JSON5 if flag is set
- Added `json5 = "0.4"` dependency

### C. File Input Option (`--file` flag)
- Direct file reading to avoid shell quoting entirely
- Precedence: `--file` overrides `--text` if both provided
- Windows-friendly path handling

## Before vs After

### Before (Poor UX)
```
$ swiftline json select --text "{a:{b:[1,2,3]}}" --path a.b[2]
Error: Failed to parse input as JSON

Caused by:
    key must be a string at line 1 column 2
```

### After (Helpful UX)
```
$ swiftline json select --text "{a:{b:[1,2,3]}}" --path a.b[2]
Error: Invalid JSON format - keys must be in double quotes

Examples of valid JSON:
  PowerShell: --text '{"a":{"b":[1,2,3]}}'
  CMD:        --text "{\"a\":{\"b\":[1,2,3]}}"
  
Alternative options:
  Use --json5 for relaxed parsing: --json5 --text "{a:{b:[1,2,3]}}"
  Use stdin: echo '{"a":{"b":[1,2,3]}}' | swiftline json select --path a.b[2]
  Use file: swiftline json select --file data.json --path a.b[2]

Original error: key must be a string at line 1 column 2
```

## Usage Examples (Windows-Specific)

### PowerShell
```powershell
# Strict JSON (works)
swiftline json select --text '{"a":{"b":[1,2,3]}}' --path a.b[2]

# Relaxed JSON5 (new)
swiftline json select --json5 --text '{a:{b:[1,2,3]}}' --path a.b[2]

# File input (recommended for complex JSON)
swiftline json select --file payload.json --path a.b[2]
```

### CMD
```cmd
REM Strict JSON with escaping
swiftline json select --text "{\"a\":{\"b\":[1,2,3]}}" --path a.b[2]

REM File input (easier)
swiftline json select --file payload.json --path a.b[2]
```

## Files Changed
- `src/cli.rs`: Added `--json5` and `--file` flags
- `src/commands/json_select.rs`: Enhanced error handling, JSON5 support, file input
- `Cargo.toml`: Added `json5 = "0.4"` dependency
- `README.md`: Updated with Windows examples
- `tests/integration_test.rs`: Added tests for new functionality

## Dependencies Added
- **json5 v0.4**: Relaxed JSON parsing (MIT license, 100KB compiled)
  - Chosen over custom normalizer for better error handling and standards compliance
  - Only used when `--json5` flag is explicitly set