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

pub fn init(endpoint: String, hostname: String, min_level: log::Level) -> Result<(), log::SetLoggerError> {
    let logger = Box::new(PapertrailLogger::new(endpoint, hostname, min_level).unwrap());
    log::set_logger(Box::leak(logger) as &dyn Log)?;
    log::set_max_level(Level::Trace.to_level_filter());
    Ok(())
}
