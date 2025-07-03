use tokio::io::{BufWriter, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_native_tls::{TlsConnector, native_tls};
use log::{Log, Record, Metadata, Level};
use chrono::Utc;

pub struct PapertrailLogger {
    hostname: String,
    sender: mpsc::Sender<String>,
    min_level: log::Level,
}

impl PapertrailLogger {
    pub fn new(endpoint: String, hostname: String, min_level: log::Level) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, mut rx) = mpsc::channel::<String>(100);
        
        let endpoint_clone = endpoint.clone();
        tokio::spawn(async move {
            let mut writer = None;
            
            // Send a test message to Papertrail
            let test_message = "<1>Test message from Rust logger\n";
            if writer.is_none() {
                // Connect to Papertrail
                let (host, _port) = endpoint_clone.split_once(':').unwrap();
                let tcp_stream = TcpStream::connect(endpoint_clone.clone()).await.unwrap();
                tcp_stream.set_nodelay(true).unwrap();
                
                let connector = TlsConnector::from(
                    native_tls::TlsConnector::builder().build()?
                );
                let stream = connector.connect(host, tcp_stream).await.unwrap();
                let stream = BufWriter::new(stream);
                writer = Some(stream);
            }
            
            if let Some(w) = &mut writer {
                if let Err(e) = w.write_all(test_message.as_bytes()).await {
                    eprintln!("Failed to send test message to Papertrail: {e:?}");
                } else if let Err(e) = w.flush().await {
                    eprintln!("Failed to flush test message to Papertrail: {e:?}");
                } else {
                    eprintln!("Successfully sent test message to Papertrail");
                }
            }
            
            while let Some(message) = rx.recv().await {
                if writer.is_none() {
                    // Connect to Papertrail
                    let (host, _port) = endpoint_clone.split_once(':').unwrap();
                    let tcp_stream = TcpStream::connect(endpoint_clone.clone()).await.unwrap();
                    tcp_stream.set_nodelay(true).unwrap();
                    
                    let connector = TlsConnector::from(
                        native_tls::TlsConnector::builder().build()?
                    );
                    let stream = connector.connect(host, tcp_stream).await.unwrap();
                    let stream = BufWriter::new(stream);
                    writer = Some(stream);
                }
                
                if let Some(w) = &mut writer {
                    if let Err(e) = w.write_all(message.as_bytes()).await {
                        eprintln!("Failed to write log: {e:?}");
                    } else if let Err(e) = w.flush().await {
                        eprintln!("Failed to flush log: {e:?}");
                    } else {
                        println!("Log sent and flushed successfully");
                    }
                }
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
        
        Ok(Self {
            hostname,
            sender: tx,
            min_level,
        })
    }
}

impl Log for PapertrailLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }
    
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let severity = match record.level() {
            Level::Error => 3,
            Level::Warn => 4,
            Level::Info => 6,
            Level::Debug => 7,
            Level::Trace => 7,
        };
        let priority = 8 + severity; // Facility 1 (user-level) + severity
        
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let message = format!("<{}>1 {} {} {} - - - [{}] {}\n", priority, now, self.hostname, record.target(), record.level(), record.args());
        
        if let Err(e) = self.sender.try_send(message) {
            eprintln!("Failed to send log message to channel: {e:?}");
        }
    }
    
    fn flush(&self) {}
}

pub fn set_logger(config: LoggerConfig) -> Result<tokio::task::JoinHandle<()>, log::SetLoggerError> {
    let (logger, handle) = AsyncLogger::new(config);
    let boxed_logger = Box::new(logger);
    log::set_logger(Box::leak(boxed_logger))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(handle)
}

// ===================== AsyncLogger Implementation =====================

use tokio::fs::{OpenOptions, create_dir_all, remove_file, read_dir};
use std::path::PathBuf;
use std::collections::VecDeque;
use std::io;

/// Log message structure for async logger
pub struct LogMessage {
    pub level: Level,
    pub target: String,
    pub body: String,
}

/// Logger configuration for both local and Papertrail logging.
#[derive(Clone, Debug)]
pub struct LoggerConfig {
    pub log_dir: PathBuf,
    pub max_file_size: u64,
    pub max_files: usize,
    pub papertrail_endpoint: Option<String>,
    pub hostname: String,
    pub enable_local: bool,
    pub enable_papertrail: bool,
}
/// AsyncLogger supports logging to both local file and Papertrail.
pub struct AsyncLogger {
    sender: mpsc::Sender<LogMessage>,
}


impl AsyncLogger {
    pub fn new(config: LoggerConfig) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<LogMessage>(100);
        let LoggerConfig {
            log_dir,
            max_file_size,
            max_files,
            papertrail_endpoint,
            hostname,
            enable_local,
            enable_papertrail,
        } = config.clone();
        let handle = tokio::spawn(async move {
            // Ensure log directory exists if local logging is enabled
            if enable_local {
                if let Err(e) = create_dir_all(&log_dir).await {
                    eprintln!("Failed to create log directory: {e:?}");
                    return;
                }
            }

            // File rotation state
            let mut log_files = VecDeque::new();
            let mut current_file = None;
            let mut current_file_size = 0u64;
            let mut file_index = 0;

            // Papertrail setup
            let mut writer = None;
            let endpoint = papertrail_endpoint;
            let mut papertrail_host = None;
            if let Some(ref ep) = endpoint {
                if let Some((host, _)) = ep.split_once(':') {
                    papertrail_host = Some(host.to_string());
                }
            }

            // Helper to rotate files
            async fn rotate_file(
                log_dir: &std::path::Path,
                log_files: &mut VecDeque<PathBuf>,
                file_index: &mut usize,
                max_files: usize,
            ) -> io::Result<(tokio::fs::File, PathBuf)> {
                let file_name = format!("log_{}.log", chrono::Local::now().format("%Y%m%dT%H%M%S%z"));
                let file_path = log_dir.join(&file_name);
                let file = OpenOptions::new().create(true).append(true).open(&file_path).await?;
                log_files.push_back(file_path.clone());
                *file_index += 1;
                // Remove old files
                while log_files.len() > max_files {
                    if let Some(old) = log_files.pop_front() {
                        let _ = remove_file(old).await;
                    }
                }
                Ok((file, file_path))
            }

            // Discover existing log files (for rotation)
            if enable_local {
                if let Ok(mut entries) = read_dir(&log_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file() && path.extension().map(|e| e == "log").unwrap_or(false) {
                            log_files.push_back(path);
                        }
                    }
                }
            }

            // Open initial log file
            if enable_local {
                let (file, _file_path) = match rotate_file(&log_dir, &mut log_files, &mut file_index, max_files).await {
                    Ok((f, p)) => (f, p),
                    Err(e) => {
                        eprintln!("Failed to open initial log file: {e:?}");
                        return;
                    }
                };
                current_file = Some(file);
                current_file_size = 0;
            }

            // Papertrail: connect if endpoint is provided
            if enable_papertrail {
                if let (Some(ref ep), Some(ref host)) = (endpoint.as_ref(), papertrail_host.as_ref()) {
                    eprintln!("[DEBUG] Connecting to Papertrail endpoint: {ep} host: {host}");
                    match TcpStream::connect(ep).await {
                        Ok(tcp_stream) => {
                            let _ = tcp_stream.set_nodelay(true);
                            match TlsConnector::from(native_tls::TlsConnector::builder().build().unwrap()).connect(host, tcp_stream).await {
                                Ok(stream) => {
                                    writer = Some(BufWriter::new(stream));
                                    eprintln!("[DEBUG] Connected to Papertrail");
                                },
                                Err(e) => eprintln!("[ERROR] Failed to connect TLS to Papertrail: {e:?}"),
                            }
                        },
                        Err(e) => eprintln!("[ERROR] Failed to connect TCP to Papertrail: {e:?}"),
                    }
                }
            }

            // Main log processing loop
            while let Some(msg) = rx.recv().await {
                // Format log line
                let now = chrono::Local::now().to_rfc3339();
                let log_line = format!("[{}][{}][{}] {}\n", now, msg.level, msg.target, msg.body);
                // Write to file
                if enable_local {
                    if let Some(f) = current_file.as_mut() {
                        if let Ok(bytes) = f.write(log_line.as_bytes()).await {
                            current_file_size += bytes as u64;
                            let _ = f.flush().await;
                        }
                    }
                    // Rotate if needed
                    if current_file_size > max_file_size {
                        if let Ok((new_file, _new_path)) = rotate_file(&log_dir, &mut log_files, &mut file_index, max_files).await {
                            current_file = Some(new_file);
                            // file_path = new_path;
                            current_file_size = 0;
                        }
                    }
                }
                // Send to Papertrail
                if enable_papertrail {
                    if let Some(w) = writer.as_mut() {
                        let pri = 8 + match msg.level {
                            Level::Error => 3,
                            Level::Warn => 4,
                            Level::Info => 6,
                            Level::Debug | Level::Trace => 7,
                        };
                        // Papertrail expects RFC5424 syslog format
                        let syslog_msg = format!("<{}>1 {} {} {} - - - [{}] {}\n", pri, now, hostname, msg.target, msg.level, msg.body);
                        eprintln!("[DEBUG] Sending to Papertrail: {}", syslog_msg.trim_end());
                        if let Err(e) = w.write_all(syslog_msg.as_bytes()).await {
                            eprintln!("[ERROR] Failed to send to Papertrail: {e:?}");
                        } else if let Err(e) = w.flush().await {
                            eprintln!("[ERROR] Failed to flush Papertrail: {e:?}");
                        } else {
                            eprintln!("[DEBUG] Sent log to Papertrail successfully");
                        }
                    }
                }
            }
        });
        (Self { sender: tx }, handle)
    }

    /// Asynchronously log a message
    pub async fn log(&self, level: Level, target: impl Into<String>, body: impl Into<String>) {
        let _ = self.sender.send(LogMessage {
            level,
            target: target.into(),
            body: body.into(),
        }).await;
    }
}

impl log::Log for AsyncLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        let _ = self.sender.try_send(LogMessage {
            level: record.level(),
            target: record.target().to_string(),
            body: format!("{}", record.args()),
        });
    }
    fn flush(&self) {}
}
