use dotenvy::dotenv;
use papertrail_logger::{LoggerConfig, set_logger};
use std::path::PathBuf;
use std::env;
use log::{info, error, warn, debug};

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env if present
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

    // Give the logger some time to write and send logs
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Optionally, gracefully shut down
    handle.await.unwrap();
}
