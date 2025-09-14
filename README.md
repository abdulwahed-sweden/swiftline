# Swiftline

Minimal, fast Rust CLI with **two essentials**:

- `http get` — GET a URL (headers, timeout, save to file, progress, pretty JSON)
- `json select` — extract a value from JSON by a simple path (supports stdin)

Windows-friendly with rustls TLS backend for reliable HTTPS.

## Install & Run

```bash
cargo build --release
cargo install --path .
# Or run directly with cargo:
cargo run -- --help
```

## Usage

### HTTP GET

```bash
# Simple GET with pretty JSON
swiftline http get https://httpbin.org/get --pretty

# With custom headers and timeout
swiftline http get https://api.github.com/user -H "Authorization: token xyz" --timeout 20

# Download file with progress
swiftline http get https://speed.hetzner.de/1MB.bin --save downloaded.bin
```

### JSON Select

```bash
# Strict JSON (cross-platform)
swiftline json select --text '{"user":{"name":"Alice","items":[1,2,3]}}' --path user.name

# File input (recommended for Windows to avoid quoting issues)
swiftline json select --file data.json --path user.items[2]

# JSON5 relaxed parsing (unquoted keys, trailing commas, etc.)
swiftline json select --json5 --text '{user: {name: "Alice", items: [1,2,3]}}' --path user.name

# From stdin (pipeline-friendly)
echo '{"a":{"b":[1,2,3]}}' | swiftline json select --path a.b[2]
curl -s https://httpbin.org/get | swiftline json select --path headers.Host
```

#### Windows-Specific Examples

**PowerShell:**
```powershell
# Using file input (easiest)
swiftline json select --file data.json --path a.b[2]

# Using stdin
echo '{"a":{"b":[1,2,3]}}' | swiftline json select --path a.b[2]

# JSON5 for relaxed syntax
swiftline json select --json5 --text '{a: {b: [1,2,3]}}' --path a.b[2]
```

**CMD:**
```cmd
REM File input recommended
swiftline json select --file data.json --path a.b[2]

REM Strict JSON with escaping (complex)
swiftline json select --text "{\"a\":{\"b\":[1,2,3]}}" --path a.b[2]
```

Path syntax:
- Objects: `user.profile.name`
- Arrays: `items[0]`, `users[2].email`
- Mixed: `data.results[0].id`

## Logging

- `-v` → info level
- `-vv` → debug level

## Structure

```
src/
├── main.rs           # Entry point, logging, CLI dispatch
├── cli.rs            # Clap CLI definitions
├── commands/
│   ├── http_get.rs   # HTTP GET with streaming & progress
│   └── json_select.rs # JSON path selection
└── helpers/
    ├── spinner.rs    # Progress spinners
    └── style.rs      # ANSI colors (Windows-compatible)
```

## Features

- 🚀 Fast compilation and runtime
- 🪟 Windows-friendly with ANSI colors  
- 🔒 Secure rustls TLS (no native TLS issues)
- 📊 Progress bars for downloads
- 🎨 Auto-colored JSON output (TTY detection)
- 📝 Enhanced error messages with shell-specific examples
- 🧪 Unit and integration tests
- 📁 File input support (`--file`) to avoid shell quoting
- 🔧 JSON5 relaxed parsing (`--json5`) for unquoted keys
- 🤝 Cross-platform compatibility (PowerShell, CMD, Bash)

## Troubleshooting

### JSON Parsing Issues

**Common Error**: "key must be a string"
```bash
# ❌ This fails (unquoted keys)
swiftline json select --text "{a:{b:[1,2,3]}}" --path a.b[2]

# ✅ Solutions:
# 1. Use strict JSON
swiftline json select --text '{"a":{"b":[1,2,3]}}' --path a.b[2]

# 2. Use JSON5 flag for relaxed parsing  
swiftline json select --json5 --text '{a:{b:[1,2,3]}}' --path a.b[2]

# 3. Use file input (best for Windows)
echo '{"a":{"b":[1,2,3]}}' > data.json
swiftline json select --file data.json --path a.b[2]
```

**PowerShell Quoting**: Complex JSON strings can be tricky in PowerShell. Use file input or stdin:

```powershell
# Easiest approach
swiftline json select --file data.json --path key

# Stdin approach  
echo '{"key":"value"}' | swiftline json select --path key
```
