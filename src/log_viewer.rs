//! `log_viewer` is the module for `LogViewer` logic.

use crate::{Args, Commands, log_entry::LogEntry, log_file::LogFile, tui::Tui};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{io, path::PathBuf, sync::mpsc::channel};

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
        let command = self.args.command.clone();
        let file_path = self.args.file_path.clone();

        match (command, file_path) {
            (Some(commands), None) => self.run_commands(&commands),
            (None, Some(file_path)) => {
                if let Err(e) = Self::run_single_file_with_tui(file_path) {
                    eprintln!("TUI error: {e}");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Application accepts commands or single-file mode only.");
                std::process::exit(1);
            }
        }
    }

    /// run the application using the provided commands
    pub fn run_commands(&self, commands: &Commands) {
        _ = commands;
        todo!(
            "Commands are not yet implemented, please use the application in single-file mode for now."
        )
    }

    /// run the application in single-file mode with TUI
    pub fn run_single_file_with_tui(file_path: String) -> io::Result<()> {
        // Initialize TUI
        let mut tui = Tui::new()?;
        tui.start()?;

        // Ensure we clean up the terminal even if there's an error
        let result = Self::run_tui_loop(file_path, &mut tui);

        // Always try to end the TUI cleanly
        let _ = tui.end();

        result
    }

    /// Main TUI loop with file watching
    fn run_tui_loop(file_path: String, tui: &mut Tui) -> io::Result<()> {
        let mut log_file = match LogFile::new(file_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error occurred while getting log file: {err}");
                return Err(std::io::Error::other(err));
            }
        };

        // Load initial log entries
        Self::load_initial_log_entries(&mut log_file, tui)?;

        // Set up file watcher
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Watcher error: {e}")))?;

        let path: PathBuf = log_file.clone().into();
        watcher
            .watch(path.as_path(), RecursiveMode::NonRecursive)
            .map_err(|e| io::Error::other(format!("Watch error: {e}")))?;

        // Use the TUI's main loop with file watching as external event handler
        tui.run_loop(|tui_ref| {
            // Check for file changes (non-blocking)
            if let Ok(Ok(Event {
                kind: EventKind::Modify(_),
                ..
            })) = rx.try_recv()
            {
                Self::update_log_entries_tui(&mut log_file, tui_ref)?;
            }
            Ok(true) // Continue running
        })
    }

    /// Load initial log entries into the TUI
    fn load_initial_log_entries(log_file: &mut LogFile, tui: &mut Tui) -> io::Result<()> {
        let entries: Vec<LogEntry> = match log_file.get_entries() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error occurred while reading initial log entries: {e}");
                return Err(io::Error::other(e));
            }
        };

        // Set initial entries (don't auto-scroll to bottom on initial load)
        tui.set_log_entries(entries);
        Ok(())
    }

    /// Update log entries in the TUI (append new entries only)
    fn update_log_entries_tui(log_file: &mut LogFile, tui: &mut Tui) -> io::Result<()> {
        let entries: Vec<LogEntry> = match log_file.get_entries() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error occurred while reading log entries: {e}");
                return Err(io::Error::other(e));
            }
        };

        // Only add new entries (this will auto-scroll to show new entries)
        tui.append_new_log_entries(entries);
        Ok(())
    }
}
