//! `log_viewer` is the module for `LogViewer` logic.

use std::{
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::{Args, Commands};

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
            (None, Some(file_path)) => Self::run_single_file(&file_path),
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
    pub fn run_single_file(file_path: &Path) {
        let file_path = validate_log_file(file_path);

        let mut current_line_number = match read_existing_lines(&file_path) {
            Ok(lines_read) => lines_read,
            Err(e) => {
                eprintln!("Error reading existing lines: {e}");
                return;
            }
        };

        let (tx, rx) = channel();

        let mut watcher = notify::recommended_watcher(tx).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1)
        });

        let watch_result = watcher.watch(&file_path, RecursiveMode::NonRecursive);

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
                if let Err(e) = read_new_lines(&file_path, &mut current_line_number) {
                    eprintln!("Error reading new lines: {e}");
                }
            }
        }
    }
}

/// Read and display all existing lines in the file
fn read_existing_lines(file_path: &PathBuf) -> std::io::Result<usize> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut line_count = 0;

    for line in reader.lines() {
        let line = line?;
        println!("{line}");
        line_count += 1;
    }

    Ok(line_count)
}

/// Read and display new lines that have been added since last read
fn read_new_lines(file_path: &PathBuf, current_line_number: &mut usize) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Skip lines we've already read and process new ones
    for (index, line) in reader.lines().enumerate() {
        if index >= *current_line_number {
            let line = line?;
            println!("{line}");
            *current_line_number = index + 1;
        }
    }

    Ok(())
}

/// validate if the file is a .log or .json type file and exists.
fn validate_log_file(file_path: &Path) -> PathBuf {
    let temp_path = file_path.to_string_lossy();
    let mut file_path = PathBuf::new();
    file_path.push(file_path.clone());

    let file_path = match shellexpand::full(&temp_path) {
        Ok(expanded) => {
            let mut path = PathBuf::new();
            path.push(expanded.to_string());
            path
        }
        Err(err) => {
            eprintln!("Error: {err}");
            handle_file_not_found(&file_path);
        }
    };

    if !file_path.exists() {
        handle_file_not_found(&file_path);
    }

    if file_path.is_dir() {
        handle_expected_file(&file_path);
    }

    match file_path.extension() {
        Some(ext) => {
            if ext != "json" && ext != "log" {
                handle_invalid_file_extension();
            }
        }
        None => {
            handle_invalid_file_extension();
        }
    }

    file_path
}

/// exits the application when a file is not found for log viewing.
fn handle_file_not_found(path: &Path) -> ! {
    eprintln!("File '{}' does not exist", path.display());
    std::process::exit(1);
}

/// exits the application when a directory was found and a file was expected.
fn handle_expected_file(path: &Path) -> ! {
    eprintln!(
        "Found directory instead of a file at path: {}.",
        path.display()
    );
    std::process::exit(1);
}

/// exits the application when the log path extension is not json or log.
fn handle_invalid_file_extension() -> ! {
    eprintln!("File found with invalid extension. Accepted log file types are '.log' and '.json'");
    std::process::exit(1);
}
