use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    pub values: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert("model".to_string(), "deepseek-v4-flash".to_string());
        values.insert(
            "base_url".to_string(),
            "https://api.deepseek.com".to_string(),
        );
        values.insert("verbose".to_string(), "false".to_string());
        values.insert("show_thinking".to_string(), "true".to_string());
        values.insert("debug".to_string(), "false".to_string());
        values.insert("language".to_string(), "zh".to_string());
        Self { values }
    }
}

impl Config {
    pub fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        // Load global config
        if let Some(global_config) = Self::load_global()? {
            for (k, v) in global_config.values {
                config.values.insert(k, v);
            }
        }

        // Load local config (overrides global)
        if let Some(local_config) = Self::load_local()? {
            for (k, v) in local_config.values {
                config.values.insert(k, v);
            }
        }

        // Environment variable overrides all
        if let Ok(api_key) = env::var("DEEPSEEK_API_KEY") {
            config.values.insert("api_key".to_string(), api_key);
        }

        Ok(config)
    }

    pub fn load_global() -> Result<Option<Self>> {
        let path = Self::global_config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read global config from {:?}", path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse global config from {:?}", path))?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn load_local() -> Result<Option<Self>> {
        let path = Self::local_config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read local config from {:?}", path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse local config from {:?}", path))?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn save_global(&self) -> Result<()> {
        let path = Self::global_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config to TOML")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write global config to {:?}", path))?;
        Ok(())
    }

    pub fn save_local(&self) -> Result<()> {
        let path = Self::local_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config to TOML")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write local config to {:?}", path))?;
        Ok(())
    }

    pub fn global_config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("com", "jjit", "jjit")
            .context("Failed to determine project directories")?;
        Ok(dirs.config_dir().join("config.toml"))
    }

    pub fn local_config_path() -> PathBuf {
        PathBuf::from(".jjit").join("config.toml")
    }

    pub fn ensure_api_key(&self) -> Result<String> {
        self.get("api_key")
            .context("DeepSeek API key not configured. Run: jjit config set api_key <your-key>")
    }
}
