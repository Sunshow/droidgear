mod app;
mod editor;
mod tui;
mod ui;

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "droidgear-tui")]
#[command(version)]
#[command(about = "DroidGear TUI (headless terminal UI)")]
struct Cli {
    /// Override $HOME for reading/writing config files (useful in containers/tests)
    #[arg(long, global = true)]
    home: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run a temporary Codex session in the current terminal and exit
    Run {
        #[command(subcommand)]
        target: RunTarget,
    },
}

#[derive(Debug, Subcommand)]
enum RunTarget {
    /// Run a Codex profile by profile id
    Codex { profile_id: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let home_dir = match cli.home {
        Some(p) => p,
        None => dirs::home_dir().context("Failed to determine $HOME")?,
    };

    match cli.command {
        Some(Command::Run { target }) => match target {
            RunTarget::Codex { profile_id } => {
                tui::run_codex_temporary_run_for_profile_id(&home_dir, &profile_id)
            }
        },
        None => {
            let mut app = app::App::new(home_dir);
            tui::run(&mut app)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, RunTarget};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn cli_parses_interactive_mode_without_subcommands() {
        let cli = Cli::parse_from(["droidgear-tui", "--home", "/tmp/demo-home"]);

        assert_eq!(cli.home, Some(PathBuf::from("/tmp/demo-home")));
        assert!(cli.command.is_none());
    }

    #[test]
    fn cli_parses_codex_run_subcommand() {
        let cli = Cli::parse_from([
            "droidgear-tui",
            "run",
            "codex",
            "profile-a",
            "--home",
            "/tmp/demo-home",
        ]);

        assert_eq!(cli.home, Some(PathBuf::from("/tmp/demo-home")));
        match cli.command {
            Some(Command::Run {
                target: RunTarget::Codex { profile_id },
            }) => {
                assert_eq!(profile_id, "profile-a");
            }
            _ => panic!("expected codex run subcommand"),
        }
    }
}
