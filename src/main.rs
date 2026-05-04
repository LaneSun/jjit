use anyhow::Result;
use clap::FromArgMatches;

use jjit::cli::{Cli, Commands};
use jjit::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    jjit::init_locale();

    let cmd = jjit::build_cli_command();
    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    let mut config = Config::load()?;

    // Override config from CLI flags
    if cli.verbose {
        config.set("verbose", "true");
    }
    if cli.show_prompt {
        config.set("show_prompt", "true");
    }
    if cli.debug {
        config.set("debug", "true");
    }
    if cli.no_thinking {
        config.set("show_thinking", "false");
    }
    if cli.no_color {
        config.set("no_color", "true");
        std::env::set_var("NO_COLOR", "1");
    }

    match cli.command {
        Commands::Commit { dry_run } => jjit::commands::commit::run(&config, dry_run).await,
        Commands::Goto { query, dry_run } => {
            jjit::commands::goto::run(&config, &query, dry_run).await
        }
        Commands::Pack { query, dry_run } => {
            jjit::commands::pack::run(&config, &query, dry_run).await
        }
        Commands::Config { action } => jjit::commands::config::run(action),
    }
}
