//! Logz is a command line log viewer application. It is designed to help
//! engineers quickly evaluate errors in their applications without a lot
//! of ceremony.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::doc_markdown)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
// ratatui depends on unicode-width and
// unicode-truncate (which depends on another version of unicode-width)
#![allow(clippy::multiple_crate_versions)]

mod log_viewer;
mod persistence;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A command line log viewer application.
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    author = "Jake Hathaway <jake.d.hathaway@gmail.com>",
    help_template = "\
{before-help}
{name}
Version: {version}
Created By: {author-with-newline}
{about-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}
"
)]
struct Args {
    /// A command
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to a single log file (when no subcommand is used)
    file_path: Option<PathBuf>,
    /// Follow mode to auto-scroll to new content
    #[arg(short, long, default_value = "false")]
    follow: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Manage registered applications
    Application {
        /// The action to perform
        #[command(subcommand)]
        action: ApplicationAction,
    },
}

/// An application action
#[derive(Subcommand, Debug, Clone)]
enum ApplicationAction {
    /// Add a new application
    Add {
        /// The name of application
        name: String,
        /// The root logging directory
        directory: String,
    },
    /// List all registered applications
    List,
    /// Remove an application
    Remove {
        /// The name of the application to remove
        name: String,
    },
}

fn main() {
    let args = Args::parse();
    let app = log_viewer::LogViewer::new(args);
    app.run();
}
