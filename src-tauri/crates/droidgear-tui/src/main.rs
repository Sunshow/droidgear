mod app;
mod editor;
mod tui;
mod ui;

use anyhow::{bail, Context};
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
    /// Run a temporary Droid/Codex/Claude session in the current terminal and exit
    Run {
        #[command(subcommand)]
        target: RunTarget,
    },
}

#[derive(Debug, Subcommand)]
enum RunTarget {
    /// Run a Codex profile by index, exact name, or profile id
    Codex {
        #[arg(long)]
        list: bool,
        profile: Option<String>,
    },
    /// Run a Claude profile by index, exact name, or profile id
    Claude {
        #[arg(long)]
        list: bool,
        #[arg(long)]
        preview: bool,
        profile: Option<String>,
    },
    /// Run a Droid settings file by name (use `global` for ~/.factory/settings.json)
    Droid {
        #[arg(long)]
        list: bool,
        settings_name: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    if droidgear_core::claude_runtime::matches_internal_launcher_args(&raw_args) {
        return droidgear_core::claude_runtime::run_internal_launcher_from_env()
            .map_err(anyhow::Error::msg);
    }

    let cli = Cli::parse();

    let home_dir = match cli.home {
        Some(p) => p,
        None => dirs::home_dir().context("Failed to determine $HOME")?,
    };

    match cli.command {
        Some(Command::Run { target }) => match target {
            RunTarget::Codex { list, profile } => {
                if list {
                    if profile.is_some() {
                        bail!("`--list` cannot be combined with a Codex target");
                    }
                    println!("{}", tui::list_codex_temporary_run_targets(&home_dir)?);
                    Ok(())
                } else {
                    let profile = profile.context(
                        "Missing Codex target. Use `droidgear-tui run codex --list` to inspect available profiles.",
                    )?;
                    tui::run_codex_temporary_run_for_selector(&home_dir, &profile)
                }
            }
            RunTarget::Claude {
                list,
                preview,
                profile,
            } => {
                if list {
                    if preview || profile.is_some() {
                        bail!("`--list` cannot be combined with other Claude run arguments");
                    }
                    println!("{}", tui::list_claude_temporary_run_targets(&home_dir)?);
                    Ok(())
                } else if preview {
                    let profile = profile.context(
                        "Missing Claude target. Use `droidgear-tui run claude --list` to inspect available profiles.",
                    )?;
                    println!(
                        "{}",
                        tui::preview_claude_temporary_run_for_selector(&home_dir, &profile)?
                    );
                    Ok(())
                } else {
                    let profile = profile.context(
                        "Missing Claude target. Use `droidgear-tui run claude --list` to inspect available profiles.",
                    )?;
                    tui::run_claude_temporary_run_for_selector(&home_dir, &profile)
                }
            }
            RunTarget::Droid {
                list,
                settings_name,
            } => {
                if list {
                    if settings_name.is_some() {
                        bail!("`--list` cannot be combined with a Droid target");
                    }
                    println!("{}", tui::list_droid_temporary_run_targets(&home_dir)?);
                    Ok(())
                } else {
                    let settings_name = settings_name.context(
                        "Missing Droid target. Use `droidgear-tui run droid --list` to inspect available settings names.",
                    )?;
                    tui::run_droid_temporary_run_for_settings_name(&home_dir, &settings_name)
                }
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
                target: RunTarget::Codex { list, profile },
            }) => {
                assert!(!list);
                assert_eq!(profile.as_deref(), Some("profile-a"));
            }
            _ => panic!("expected codex run subcommand"),
        }
    }

    #[test]
    fn cli_parses_droid_run_subcommand() {
        let cli = Cli::parse_from(["droidgear-tui", "run", "droid", "global"]);

        match cli.command {
            Some(Command::Run {
                target:
                    RunTarget::Droid {
                        list,
                        settings_name,
                    },
            }) => {
                assert!(!list);
                assert_eq!(settings_name.as_deref(), Some("global"));
            }
            _ => panic!("expected droid run subcommand"),
        }
    }

    #[test]
    fn cli_parses_codex_list_subcommand() {
        let cli = Cli::parse_from(["droidgear-tui", "run", "codex", "--list"]);

        match cli.command {
            Some(Command::Run {
                target: RunTarget::Codex { list, profile },
            }) => {
                assert!(list);
                assert!(profile.is_none());
            }
            _ => panic!("expected codex list subcommand"),
        }
    }

    #[test]
    fn cli_parses_droid_list_subcommand() {
        let cli = Cli::parse_from(["droidgear-tui", "run", "droid", "--list"]);

        match cli.command {
            Some(Command::Run {
                target:
                    RunTarget::Droid {
                        list,
                        settings_name,
                    },
            }) => {
                assert!(list);
                assert!(settings_name.is_none());
            }
            _ => panic!("expected droid list subcommand"),
        }
    }

    #[test]
    fn cli_parses_claude_run_subcommand() {
        let cli = Cli::parse_from([
            "droidgear-tui",
            "run",
            "claude",
            "profile-a",
            "--home",
            "/tmp/demo-home",
        ]);

        assert_eq!(cli.home, Some(PathBuf::from("/tmp/demo-home")));
        match cli.command {
            Some(Command::Run {
                target:
                    RunTarget::Claude {
                        list,
                        preview,
                        profile,
                    },
            }) => {
                assert!(!list);
                assert!(!preview);
                assert_eq!(profile.as_deref(), Some("profile-a"));
            }
            _ => panic!("expected claude run subcommand"),
        }
    }

    #[test]
    fn cli_parses_claude_list_subcommand() {
        let cli = Cli::parse_from(["droidgear-tui", "run", "claude", "--list"]);

        match cli.command {
            Some(Command::Run {
                target:
                    RunTarget::Claude {
                        list,
                        preview,
                        profile,
                    },
            }) => {
                assert!(list);
                assert!(!preview);
                assert!(profile.is_none());
            }
            _ => panic!("expected claude list subcommand"),
        }
    }

    #[test]
    fn cli_parses_claude_preview_subcommand() {
        let cli = Cli::parse_from(["droidgear-tui", "run", "claude", "--preview", "profile-a"]);

        match cli.command {
            Some(Command::Run {
                target:
                    RunTarget::Claude {
                        list,
                        preview,
                        profile,
                    },
            }) => {
                assert!(!list);
                assert!(preview);
                assert_eq!(profile.as_deref(), Some("profile-a"));
            }
            _ => panic!("expected claude preview subcommand"),
        }
    }
}
