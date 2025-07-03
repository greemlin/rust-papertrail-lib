# Rust Papertrail Logger

[![Crates.io](https://img.shields.io/crates/v/rust-papertrail-lib)](https://crates.io/crates/rust-papertrail-lib)
[![Docs.rs](https://docs.rs/rust-papertrail-lib/badge.svg)](https://docs.rs/rust-papertrail-lib)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, fully asynchronous logger for Rust with dual output to local files and [Papertrail](https://www.papertrail.com/). Integrates with the standard `log` macros and supports advanced features like file rotation, retention, and runtime configuration via environment variables.

---

## Features

- **Async logging** with Tokio
- **Dual output:** Local file + Papertrail (TCP+TLS)
- **File rotation** by size and retention by count
- **Syslog-compliant** RFC5424 messages for Papertrail
- **Configurable via `.env` or environment**
- **Easy integration with `log` macros** (`info!`, `warn!`, etc.)
- **Hot enable/disable** of local and remote logging
- **Custom hostname and log directory**
- **Built-in debug output for troubleshooting**

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
papertrail_logger = "1.0.0" # Check crates.io for latest
log = "0.4"
dotenvy = "0.15" # For .env support (optional, but recommended)
```

---

## Quick Start

### 1. Create a `.env` file (recommended)

```
# Enable/disable local logging
ENABLE_LOCAL_LOG=true

# Enable/disable Papertrail logging
ENABLE_PAPERTRAIL_LOG=true

# Directory for local log files
LOG_DIR=logs

# Max file size (bytes) before rotating
MAX_FILE_SIZE=1048576

# Max number of log files to keep
MAX_FILES=5

# Papertrail endpoint (host:port)
PAPERTRAIL_ENDPOINT=logsX.papertrailapp.com:XXXXX

# Hostname for syslog messages
HOSTNAME=my-app-host
```

### 2. Use in your app

```rust
use dotenvy::dotenv; // or dotenv::dotenv if you prefer
use papertrail_logger::{LoggerConfig, set_logger};
use std::path::PathBuf;
use std::env;
use log::{info, warn, error, debug};

#[tokio::main]
async fn main() {
    dotenv().ok(); // Loads .env if present
    let log_dir = env::var("LOG_DIR").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("logs"));
    let max_file_size = env::var("MAX_FILE_SIZE").ok().and_then(|s| s.parse().ok()).unwrap_or(1024 * 1024);
    let max_files = env::var("MAX_FILES").ok().and_then(|s| s.parse().ok()).unwrap_or(5);
    let papertrail_endpoint = env::var("PAPERTRAIL_ENDPOINT").ok();
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".to_string());
    let enable_local = env::var("ENABLE_LOCAL_LOG").map(|v| v == "true" || v == "1").unwrap_or(true);
    let enable_papertrail = env::var("ENABLE_PAPERTRAIL_LOG").map(|v| v == "true" || v == "1").unwrap_or(true);

    let config = LoggerConfig {
        log_dir,
        max_file_size,
        max_files,
        papertrail_endpoint,
        hostname,
        enable_local,
        enable_papertrail,
    };

    let handle = set_logger(config).expect("Failed to set logger");

    info!("This is an info message");
    warn!("This is a warning message");
    debug!("This is a debug message");
    error!("This is an error message");

    // Give the logger some time to flush
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    handle.await.unwrap();
}
```

---

## Configuration Reference

| Variable                | Description                      | Example                          |
|-------------------------|----------------------------------|----------------------------------|
| `ENABLE_LOCAL_LOG`      | Enable local file logging         | `true` / `false`                 |
| `ENABLE_PAPERTRAIL_LOG` | Enable Papertrail logging         | `true` / `false`                 |
| `LOG_DIR`               | Directory for local log files     | `logs`                           |
| `MAX_FILE_SIZE`         | Max file size in bytes (rotation) | `1048576`                        |
| `MAX_FILES`             | Max rotated log files to keep     | `5`                              |
| `PAPERTRAIL_ENDPOINT`   | Papertrail endpoint host:port     | `logsX.papertrailapp.com:XXXXX`  |
| `HOSTNAME`              | Hostname for syslog messages      | `my-app-host`                    |

- All variables are optional; defaults are provided if missing.
- You may also configure via a `LoggerConfig` struct directly.

---

## Features & Usage

- **Log Macros:** Use any of `info!`, `warn!`, `error!`, `debug!`, `trace!` from the standard `log` crate.
- **Async:** All logging is non-blocking and handled in a background Tokio task.
- **File Rotation:** Local logs are rotated by size and oldest files are deleted when `MAX_FILES` is exceeded.
- **Syslog Format:** Remote logs are sent in RFC5424 syslog format over TLS.
- **Debug Output:** Internal debug/error messages are printed to stderr for troubleshooting.

---

## Troubleshooting

- If Papertrail logs do not appear:
  - Double-check `PAPERTRAIL_ENDPOINT` and port.
  - Ensure outbound TCP is allowed to Papertrail.
  - Watch for `[DEBUG]`/`[ERROR]` messages in your terminal.
  - Try running with `RUST_LOG=debug` for more verbose output.
- If local logs do not appear:
  - Check `LOG_DIR` exists and is writable.
  - Verify `ENABLE_LOCAL_LOG` is set to `true`.

---

## Minimum Supported Rust Version

- Rust 1.60+
- Requires Tokio async runtime

---

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please:
- Follow Rust coding standards
- Add tests for new features
- Document public APIs

---

## Acknowledgements

- [Papertrail](https://www.papertrail.com/) for their logging service
- [Tokio](https://tokio.rs/) for async runtime
- [dotenvy](https://crates.io/crates/dotenvy) for .env support


### Advanced Configuration

You can construct a `LoggerConfig` manually for custom setups, or toggle logging backends at runtime:

```rust
use papertrail_logger::{LoggerConfig, set_logger};
use std::path::PathBuf;

let config = LoggerConfig {
    log_dir: PathBuf::from("custom-logs"),
    max_file_size: 512 * 1024, // 512KB
    max_files: 10,
    papertrail_endpoint: Some("logsX.papertrailapp.com:XXXXX".to_string()),
    hostname: "custom-hostname".to_string(),
    enable_local: true,
    enable_papertrail: false, // Start with only local logging
};
let handle = set_logger(config).expect("Failed to set logger");
// ...
// You can later enable Papertrail by updating config and restarting the logger task.
```

## Configuration

### Environment Variables

| Variable                | Description                      | Example                          |
|-------------------------|----------------------------------|----------------------------------|
| `ENABLE_LOCAL_LOG`      | Enable local file logging         | `true` / `false`                 |
| `ENABLE_PAPERTRAIL_LOG` | Enable Papertrail logging         | `true` / `false`                 |
| `LOG_DIR`               | Directory for local log files     | `logs`                           |
| `MAX_FILE_SIZE`         | Max file size in bytes (rotation) | `1048576`                        |
| `MAX_FILES`             | Max rotated log files to keep     | `5`                              |
| `PAPERTRAIL_ENDPOINT`   | Papertrail endpoint host:port     | `logsX.papertrailapp.com:XXXXX`  |
| `HOSTNAME`              | Hostname for syslog messages      | `my-app-host`                    |
| `RUST_LOG`              | Minimum log level                 | `debug`                          |

### Log Levels

| Rust Level | Syslog Severity | Description |
|------------|-----------------|-------------|
| Error | 3 (Error) | Critical failures |
| Warn | 4 (Warning) | Potential issues |
| Info | 6 (Informational) | Normal operation |
| Debug | 7 (Debug) | Debug messages |
| Trace | 7 (Debug) | Detailed traces |

## Testing

You can write async integration tests using Tokio and the logger's config-driven API. Example:

```rust
use papertrail_logger::{LoggerConfig, set_logger};
use std::path::PathBuf;
use log::info;

#[tokio::test]
async fn test_logger_init_and_log() {
    let config = LoggerConfig {
        log_dir: PathBuf::from("test-logs"),
        max_file_size: 1024 * 1024,
        max_files: 2,
        papertrail_endpoint: None, // Avoid real remote logging in tests
        hostname: "test-host".to_string(),
        enable_local: true,
        enable_papertrail: false,
    };
    let handle = set_logger(config).expect("Failed to set logger");
    info!("Test log message");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    handle.abort(); // Stop background task
    // Optionally, check that a log file was created in test-logs/
}
```

- Use `tokio::test` for async tests.
- Set `papertrail_endpoint: None` to avoid sending logs remotely during tests.
- Use a unique log directory per test to avoid conflicts.
- Use `handle.abort()` to stop the logger background task at the end of your test.

Run tests with:
```bash
cargo test -- --nocapture
```

## Best Practices

1. **Use appropriate log levels**:
   - Reserve `error!` for critical failures
   - Use `debug!` and `trace!` for troubleshooting
2. **Include useful context**:
   ```rust
   log::info!("User {} authenticated", user_id);
   ```
3. **Handle secrets carefully**:
   ```rust
   log::debug!("Processing transaction {}", transaction_id);
   // NOT: log::info!("Credit card: {}", card_number);
   ```

## Contributing

Contributions are welcome! Please follow:
1. Rust coding standards
2. Comprehensive test coverage
3. Detailed documentation

## License

MIT License - see [LICENSE](LICENSE) for details.
