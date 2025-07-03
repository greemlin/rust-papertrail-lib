use std::fs::File;
use std::io::{Write, BufWriter};
use std::sync::Mutex;
use log::Record;
use chrono::Local;

pub struct FileLogger {
    folder: String,
    max_file_size: u64,
    max_files: u32,
    current_file: Mutex<Option<BufWriter<File>>>,
    current_size: Mutex<usize>,
}

impl FileLogger {
    pub fn new(folder: String, max_file_size: u64, max_files: u32) -> Self {
        Self {
            folder,
            max_file_size,
            max_files,
            current_file: Mutex::new(None),
            current_size: Mutex::new(0),
        }
    }

    pub fn log(&self, record: &Record) {
        let message = format!("{}", record.args());
        let message_size = message.len();

        // Lock current_size for the duration of this block
        let mut current_size_guard = self.current_size.lock().unwrap();
        
        if *current_size_guard + message_size > self.max_file_size as usize {
            self.rotate_files();
            *current_size_guard = 0;
        }

        // Lock current_file for the duration of this block
        let mut current_file_guard = self.current_file.lock().unwrap();
        
        if let Some(file) = &mut *current_file_guard {
            if writeln!(file, "{} - {} - {}", 
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ).is_ok() {
                *current_size_guard += message_size;
            }
        }
    }

    fn rotate_files(&self) {
        // Create the folder if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&self.folder) {
            eprintln!("Failed to create log folder: {}", e);
            return;
        }

        // Generate a new file name with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let file_path = format!("{}/{}.log", self.folder, timestamp);

        // Open the file and update state
        match File::create(&file_path) {
            Ok(file) => {
                let mut current_file = self.current_file.lock().unwrap();
                *current_file = Some(BufWriter::new(file));
            }
            Err(e) => eprintln!("Failed to open log file: {}", e),
        }

        self.clean_old_files();
    }

    fn clean_old_files(&self) {
        // List all files in the folder
        if let Ok(entries) = std::fs::read_dir(&self.folder) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .collect();

            // Sort by creation time (oldest first)
            files.sort_by_key(|f| f.metadata().and_then(|m| m.created()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));

            // Delete the oldest files if we exceed max_files
            if files.len() > self.max_files as usize {
                for file in files.iter().take(files.len() - self.max_files as usize) {
                    if let Err(e) = std::fs::remove_file(file) {
                        eprintln!("Failed to remove old log file: {}", e);
                    }
                }
            }
        }
    }
}
