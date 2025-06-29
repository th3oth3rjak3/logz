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
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to a single log file (when no subcommand is used)
    file_path: Option<String>,
    /// Follow mode to auto-scroll to new content
    #[arg(short, long, default_value = "false")]
    follow: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage registered applications
    Application {
        #[command(subcommand)]
        action: ApplicationAction,
    },
}

#[derive(Subcommand, Debug)]
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

    // for _ in 0..args.count {
    //     println!("Hello {}!", args.name);
    // }
}
