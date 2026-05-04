pub mod cli;
pub mod commands;
pub mod config;
pub mod jj_util;
pub mod llm;
pub mod output;

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
