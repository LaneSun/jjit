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
                println!("{}", crate::t!("messages.config_set_global", arg1 = key, arg2 = value));
            } else {
                config.save_local()?;
                println!("{}", crate::t!("messages.config_set_local", arg1 = key, arg2 = value));
            }
        }
        ConfigAction::Get { key } => {
            let config = Config::load()?;
            match config.get(&key) {
                Some(value) => println!("{}", value),
                None => println!("{}", crate::t!("messages.config_not_set")),
            }
        }
        ConfigAction::List => {
            let config = Config::load()?;
            for (key, value) in &config.values {
                // Don't print API key in full
                if key == "api_key" && value.len() > 8 {
                    let masked = &value[..8];
                    println!(
                        "{}",
                        crate::t!("messages.api_key_masked", arg1 = key, arg2 = masked)
                    );
                } else {
                    println!("{} = {}", key, value);
                }
            }
        }
    }

    Ok(())
}
