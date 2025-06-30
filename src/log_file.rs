//! `log_file` is a module that contains abstractions for a `LogFile` type.

use std::path::{Path, PathBuf};

use crate::log_entry::LogEntry;

/// `LogFileExtension` contains the supported extensions for log files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogFileExtension {
    /// `Log` represents the plaintext ".log" extension.
    Log,
    /// `Json` represents JavaScript Object Notation.
    Json,
}

/// `LogFile` is a file that contains application logs.
#[derive(Debug, Clone)]
pub struct LogFile {
    /// `path` is the path to the file.
    path: String,

    /// `extension` is the type of extension this file has.
    extension: LogFileExtension,

    /// `current_line` is the current cursor position of the log file.
    current_line: usize,
}

impl LogFile {
    /// Create a new `LogFile` for the file located at the path.
    pub fn new(path: String) -> Result<Self, String> {
        let mut log = Self {
            path,
            extension: LogFileExtension::Log,
            current_line: 0,
        };

        log.expand_path()?;
        log.check_exists()?;
        log.check_is_file()?;
        log.set_extension()?;

        Ok(log)
    }

    /// `expand_path` is used to change relative paths with leading `~` into a user directory.
    fn expand_path(&mut self) -> Result<(), String> {
        let path = self.path.clone();
        shellexpand::full(&path)
            .map(|expanded| {
                self.path = expanded.into_owned();
            })
            .map_err(|e| e.to_string())
    }

    /// `check_exists` ensures that the file is valid at least at `LogFile` creation time.
    fn check_exists(&self) -> Result<(), String> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Err("File not found".to_owned());
        }

        Ok(())
    }

    /// `check_is_file` ensures that the associated path is to a real file.
    fn check_is_file(&self) -> Result<(), String> {
        let pb: PathBuf = self.clone().into();
        if pb.is_dir() {
            return Err("expected file, but found directory instead".into());
        }

        Ok(())
    }

    /// `set_extension` determines the file extension and sets the internal representation.
    fn set_extension(&mut self) -> Result<(), String> {
        let path = Path::new(&self.path);
        let ext = match path.extension() {
            Some(os_str) => {
                let extension = os_str.to_string_lossy().to_string();
                match extension.as_str() {
                    "json" => LogFileExtension::Json,
                    "log" => LogFileExtension::Log,
                    _ => return Err("extension not supported".into()),
                }
            }
            _ => return Err("extension not supported".into()),
        };

        self.extension = ext;

        Ok(())
    }

    /// `get_entries` gets all of the current log file entries in the file.
    /// After iteration of all lines, it updates the internal state of the log file to
    /// know where we left off for the next read.
    pub fn get_entries(&mut self) -> Result<Vec<LogEntry>, String> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        let path = self.path.clone();
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut entries: Vec<LogEntry> = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            if i > self.current_line {
                let line = line.map_err(|e| e.to_string())?;
                entries.push(LogEntry::new(i, line));
            }
        }

        self.current_line = entries
            .iter()
            .max_by_key(|ent| ent.line)
            .map(|ent| ent.line)
            .unwrap_or_default();

        Ok(entries)
    }
}

impl From<LogFile> for PathBuf {
    fn from(value: LogFile) -> Self {
        let mut pb = Self::new();
        pb.push(value.path);
        pb
    }
}
