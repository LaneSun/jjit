use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, GotoOutput, LlmOutput};

pub async fn run(config: &Config, query: &str, dry_run: bool) -> Result<()> {
    let verbose = config.get("verbose") == Some("true".to_string());
    let show_prompt = config.get("show_prompt") == Some("true".to_string());
    let show_thinking = config.get("show_thinking") != Some("false".to_string());
    let debug = config.get("debug") == Some("true".to_string());

    // Get commit history
    let log_entries = jj_util::get_log_all().context(crate::t!("errors.config_read"))?;

    let log_text = log_entries
        .iter()
        .map(|entry| {
            format!(
                "Commit ID: {}
Author: {} <{}>
Description: {}
Date: {}
",
                entry.commit_id,
                entry.author_name,
                entry.author_email,
                entry.description,
                entry.author_date
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let api_key = config.ensure_api_key()?;
    let base_url = config
        .get("base_url")
        .unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let model = config
        .get("model")
        .unwrap_or_else(|| "deepseek-v4-flash".to_string());

    let client = LlmClient::new(api_key, base_url, model);

    let user_prompt = format!("User request: {}\n\nCommit history:\n{}", query, log_text);

    let system_prompt = add_language_hint(crate::t!("prompts.goto_system").as_ref(), config);

    let response = client
        .chat(
            &system_prompt,
            &user_prompt,
            verbose,
            show_prompt,
            show_thinking,
            debug,
        )
        .await
        .context(crate::t!("errors.llm_failed"))?;

    let output = parse_xml_output(&response).context(crate::t!("errors.parse_xml"))?;

    match output {
        LlmOutput::Goto(GotoOutput { summary, target }) => {
            println!("{}", summary);
            if dry_run {
                println!("{}", crate::t!("messages.goto_dry_run", arg = target));
            } else {
                jj_util::new_commit(&target).context(crate::t!("errors.config_read"))?;
                println!("{}", crate::t!("messages.goto_success", arg = target));
            }
        }
        LlmOutput::Reply(reply) => {
            println!("{}", reply.message);
        }
        _ => {
            return Err(anyhow::anyhow!(crate::t!("errors.unexpected_response")));
        }
    }

    Ok(())
}

fn add_language_hint(prompt: &str, config: &Config) -> String {
    let lang = config.resolve_language();
    if !lang.is_empty() && lang != "en" {
        format!("{}\n\nYou MUST respond in {} language.", prompt, lang)
    } else {
        prompt.to_string()
    }
}
