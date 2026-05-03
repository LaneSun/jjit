use anyhow::Result;

use crate::cli::ConfigAction;
use crate::config::Config;

pub fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Set { key, value, global } => {
            let mut config = Config::default();

            if global {
                // Load existing global config
                if let Some(existing) = Config::load_global()? {
                    for (k, v) in existing.values {
                        config.values.insert(k, v);
                    }
                }
            } else {
                // Load existing local config
                if let Some(existing) = Config::load_local()? {
                    for (k, v) in existing.values {
                        config.values.insert(k, v);
                    }
                }
            }

            config.set(&key, &value);

            if global {
                config.save_global()?;
                println!("Set {} = {} (global)", key, value);
            } else {
                config.save_local()?;
                println!("Set {} = {} (local)", key, value);
            }
        }
        ConfigAction::Get { key } => {
            let config = Config::load()?;
            match config.get(&key) {
                Some(value) => println!("{}", value),
                None => println!("(not set)"),
            }
        }
        ConfigAction::List => {
            let config = Config::load()?;
            for (key, value) in &config.values {
                // Don't print API key in full
                if key == "api_key" && value.len() > 8 {
                    println!("{} = {}****", key, &value[..8]);
                } else {
                    println!("{} = {}", key, value);
                }
            }
        }
    }

    Ok(())
}
