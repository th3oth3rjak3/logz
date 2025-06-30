//! `log_viewer` is the module for `LogViewer` logic.

use std::{path::PathBuf, sync::mpsc::channel};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::{Args, Commands, log_entry::LogEntry, log_file::LogFile};

/// `LogViewer` manages the viewing of log files.
#[derive(Debug)]
pub struct LogViewer {
    /// `args` are the input arguments to the command line
    /// and are used by the `LogViewer` to determine how
    /// best to display the log files.
    args: Args,
}

impl LogViewer {
    /// Create a new `LogViewer`
    pub const fn new(args: Args) -> Self {
        Self { args }
    }

    /// run the application.
    pub fn run(&self) {
        _ = self.args;
        let command = self.args.command.clone();
        let file_path = self.args.file_path.clone();
        match (command, file_path) {
            (Some(commands), None) => self.run_commands(&commands),
            (None, Some(file_path)) => Self::run_single_file(file_path),
            _ => {
                eprintln!("Application accepts commands or single-file mode only.");
                std::process::exit(1);
            }
        }
    }

    /// run the application using the provided commands
    pub fn run_commands(&self, commands: &Commands) {
        _ = self;
        _ = commands;
        todo!(
            "Commands are not yet implemented, please use the application in single-file mode for now."
        )
    }

    /// run the application in single-file mode
    pub fn run_single_file(file_path: String) {
        let mut log_file = match LogFile::new(file_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error occurred while getting log file: {err}");
                std::process::exit(1);
            }
        };

        let entries: Vec<LogEntry> = match log_file.get_entries() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error occurred while reading log entries: {e}");
                std::process::exit(1);
            }
        };

        for entry in &entries {
            // TUI::print(entry)
            println!("{}", entry.content);
        }

        let (tx, rx) = channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1)
        });

        let path: PathBuf = log_file.clone().into();

        let watch_result = watcher.watch(path.as_path(), RecursiveMode::NonRecursive);

        if let Err(e) = watch_result {
            eprintln!("Error: {e}");
            return;
        }

        loop {
            if let Ok(Ok(Event {
                kind: EventKind::Modify(_),
                ..
            })) = rx.recv()
            {
                let entries: Vec<LogEntry> = match log_file.get_entries() {
                    Ok(entries) => entries,
                    Err(e) => {
                        eprintln!("Error occurred while reading log entries: {e}");
                        std::process::exit(1);
                    }
                };

                for entry in &entries {
                    // TUI::print(entry)
                    println!("{}", entry.content);
                }
            }
        }
    }
}
