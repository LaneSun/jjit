pub mod cli;
pub mod commands;
pub mod config;
pub mod jj_util;
pub mod llm;
pub mod output;

use clap::CommandFactory;

// Re-export rust_i18n macro for use across the crate
pub use rust_i18n::t;

// Initialize i18n
rust_i18n::i18n!("locales", fallback = "en");

/// Initialize locale based on environment
pub fn init_locale() {
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "en".to_string());

    // Extract language code from LANG (e.g., "zh_CN.UTF-8" -> "zh")
    let lang = locale
        .split('.')
        .next()
        .and_then(|s| s.split('_').next())
        .unwrap_or("en");

    rust_i18n::set_locale(lang);
}

/// Build CLI command with localized about texts
pub fn build_cli_command() -> clap::Command {
    let mut cmd = crate::cli::Cli::command();
    cmd = cmd.about(crate::t!("about"));

    // Override subcommand about texts
    let subcommands = [
        ("commit", crate::t!("commit.about")),
        ("goto", crate::t!("goto.about")),
        ("pack", crate::t!("pack.about")),
        ("config", crate::t!("config.about")),
    ];

    for (name, about) in subcommands {
        cmd = cmd.mut_subcommand(name, |c| c.about(about));
    }

    // Override config subcommand about texts
    cmd = cmd.mut_subcommand("config", |c| {
        c.about(crate::t!("config.about"))
            .mut_subcommand("set", |c| c.about(crate::t!("config.set.about")))
            .mut_subcommand("get", |c| c.about(crate::t!("config.get.about")))
            .mut_subcommand("list", |c| c.about(crate::t!("config.list.about")))
    });

    cmd
}
