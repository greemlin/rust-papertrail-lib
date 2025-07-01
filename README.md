# Rust Papertrail Logger

[![Crates.io](https://img.shields.io/crates/v/rust-papertrail-lib)](https://crates.io/crates/rust-papertrail-lib)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Rust library for asynchronous logging to [Papertrail](https://www.papertrail.com/). Built with reliability and ease of integration in mind.

## Features

- **Asynchronous logging** with Tokio runtime
- **TLS-secured** connections to Papertrail
- **Syslog-compliant** message formatting
- **Configurable log levels** (Error, Warn, Info, Debug, Trace)
- **Environment-based configuration**
- **Automatic reconnection** logic
- **Rich metadata** including timestamps and hostnames

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
papertrail-logger = "0.3.0"  # Check crates.io for latest version
dotenv = "0.15.0"  # For .env configuration
```

## Usage

### Basic Setup

```rust
use dotenv;
use rust_papertrail_lib::init;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // Load .env file
    init(
        std::env::var("PAPERTRAIL_ENDPOINT").expect("PAPERTRAIL_ENDPOINT must be set"),
        std::env::var("PAPERTRAIL_HOSTNAME").expect("PAPERTRAIL_HOSTNAME must be set"),
        log::Level::Info // Minimum log level
    ).expect("Failed to initialize logger");
    
    // Log at different levels
    log::error!("Critical application error");
    log::warn!("Potential issue detected");
    log::info!("Application started successfully");
    log::debug!("Debugging information");
    log::trace!("Detailed trace information");
}
```

### Advanced Configuration

```rust
// Custom initialization with error handling
match init(
    std::env::var("PAPERTRAIL_ENDPOINT")?,
    std::env::var("PAPERTRAIL_HOSTNAME")?,
    log::Level::Debug
) {
    Ok(_) => log::info!("Logger initialized successfully"),
    Err(e) => log::error!("Logger initialization failed: {}", e),
}
```

## Configuration

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `PAPERTRAIL_ENDPOINT` | Papertrail destination | `logs.papertrailapp.com:12345` |
| `PAPERTRAIL_HOSTNAME` | Host identifier in logs | `production-server-01` |
| `RUST_LOG` | Minimum log level | `debug` |

### Log Levels

| Rust Level | Syslog Severity | Description |
|------------|-----------------|-------------|
| Error | 3 (Error) | Critical failures |
| Warn | 4 (Warning) | Potential issues |
| Info | 6 (Informational) | Normal operation |
| Debug | 7 (Debug) | Debug messages |
| Trace | 7 (Debug) | Detailed traces |

## Testing

```rust
#[test]
fn test_logger_init() {
    std::env::set_var("PAPERTRAIL_ENDPOINT", "mock:1234");
    std::env::set_var("PAPERTRAIL_HOSTNAME", "test-host");
    
    assert!(init("mock:1234".into(), "test-host".into(), log::Level::Info).is_ok());
}
```

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
