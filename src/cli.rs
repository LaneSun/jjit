use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jjit")]
#[command(about = "AI-powered jj version management tool")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(long, global = true, help = "Show prompts sent to LLM for debugging")]
    pub show_prompt: bool,

    #[arg(long, global = true, help = "Show detailed debug information")]
    pub debug: bool,

    #[arg(long, global = true, help = "Hide LLM thinking process")]
    pub no_thinking: bool,

    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Check changes and auto-generate commit message via LLM")]
    Commit {
        #[arg(long, help = "Preview the commit message without creating it")]
        dry_run: bool,
    },

    #[command(about = "Find and checkout target commit via LLM")]
    Goto {
        #[arg(help = "Description of the target commit")]
        query: String,
        #[arg(long, help = "Preview the target without checking out")]
        dry_run: bool,
    },

    #[command(about = "Combine commits via LLM")]
    Pack {
        #[arg(help = "Description of commits to pack")]
        query: String,
        #[arg(long, help = "Preview the pack operation without executing")]
        dry_run: bool,
    },

    #[command(about = "Manage configuration")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Set a configuration value")]
    Set {
        #[arg(help = "Configuration key")]
        key: String,
        #[arg(help = "Configuration value")]
        value: String,
        #[arg(long, help = "Set in global config")]
        global: bool,
    },

    #[command(about = "Get a configuration value")]
    Get {
        #[arg(help = "Configuration key")]
        key: String,
    },

    #[command(about = "List all configuration values")]
    List,
}
