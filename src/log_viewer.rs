//! `log_viewer` is the module for `LogViewer` logic.

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
            (Some(commands), None) => self.run_commands(&commands, self.args.follow),
            (None, Some(file_path)) => self.run_single_file(&file_path, self.args.follow),
            _ => {
                eprintln!("Application accepts commands or single-file mode only.");
                std::process::exit(1);
            }
        }
    }

    /// run the application using the provided commands
    pub const fn run_commands(&self, commands: &Commands, follow: bool) {
        _ = self;
        _ = commands;
        _ = follow;
    }

    /// run the application in single-file mode
    pub const fn run_single_file(&self, file_path: &String, follow: bool) {
        _ = self;
        _ = file_path;
        _ = follow;
    }
}
