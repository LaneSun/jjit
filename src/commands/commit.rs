use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, CommitOutput, LlmOutput};

pub async fn run(config: &Config, dry_run: bool) -> Result<()> {
    let verbose = config.get("verbose") == Some("true".to_string());
    let show_prompt = config.get("show_prompt") == Some("true".to_string());
    let show_thinking = config.get("show_thinking") != Some("false".to_string());
    let debug = config.get("debug") == Some("true".to_string());

    // Check if there are changes
    if !jj_util::has_changes().context(crate::t!("errors.config_read"))? {
        println!("{}", crate::t!("errors.no_changes"));
        return Ok(());
    }

    // Get diff information
    let diff_summary = jj_util::get_diff_summary().context(crate::t!("errors.config_read"))?;
    let diff = jj_util::get_diff().context(crate::t!("errors.config_read"))?;

    let api_key = config.ensure_api_key()?;
    let base_url = config
        .get("base_url")
        .unwrap_or_else(|| "https://api.deepseek.com".to_string());
    let model = config
        .get("model")
        .unwrap_or_else(|| "deepseek-v4-flash".to_string());

    let client = LlmClient::new(api_key, base_url, model);

    let user_prompt = format!(
        "Files changed ({})\n{}\n\nDiff:\n{}",
        diff_summary.len(),
        diff_summary
            .iter()
            .map(|f| format!("  {:?} - {}", f.kind, f.path.display()))
            .collect::<Vec<_>>()
            .join("\n"),
        diff
    );

    let system_prompt = add_language_hint(crate::t!("prompts.commit_system").as_ref(), config);

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
        LlmOutput::Commit(CommitOutput { message, body, .. }) => {
            let full_message = if let Some(body_text) = body {
                format!("{}\n\n{}", message, body_text)
            } else {
                message
            };

            if dry_run {
                println!("{}", crate::t!("messages.commit_dry_run", arg = full_message));
            } else {
                jj_util::commit(&full_message).context(crate::t!("errors.config_read"))?;
                println!("{}", crate::t!("messages.commit_success", arg = full_message));
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
