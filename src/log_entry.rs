//! `log_entry` represents a row in a `LogFile`

/// `LogEntry` represents a row in a `LogFile`.
pub struct LogEntry {
    /// `line` is the line number where the content was found in the log file.
    pub line: usize,
    /// `content` is the actual string content of the log message.
    pub content: String,
}

impl LogEntry {
    /// Create a new `LogEntry`
    pub const fn new(line: usize, content: String) -> Self {
        Self { line, content }
    }
}
