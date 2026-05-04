use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, LlmOutput, PackOutput};

pub async fn run(config: &Config, query: &str, dry_run: bool) -> Result<()> {
    let verbose = config.get("verbose") == Some("true".to_string());
    let show_prompt = config.get("show_prompt") == Some("true".to_string());
    let show_thinking = config.get("show_thinking") != Some("false".to_string());
    let debug = config.get("debug") == Some("true".to_string());

    // Get commit history
    let log_entries = jj_util::get_log_all().context(crate::t!("errors.config_read"))?;

    // Filter out root and empty commits before sending to LLM
    let meaningful_entries: Vec<_> = log_entries
        .iter()
        .filter(|e| !e.empty && e.commit_id != "0000000000000000000000000000000000000000")
        .collect();

    let log_text = meaningful_entries
        .iter()
        .map(|entry| {
            format!(
                "Commit ID: {}\nAuthor: {} <{}>\nDescription: {}\nDate: {}\n",
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

    let system_prompt = add_language_hint(crate::t!("prompts.pack_system").as_ref(), config);

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
        LlmOutput::Pack(PackOutput {
            summary,
            commits,
            message,
            body,
        }) => {
            println!("{}", summary);

            if commits.is_empty() {
                return Err(anyhow::anyhow!(crate::t!("errors.pack_empty")));
            }

            // Filter out empty commits (like working copy)
            let non_empty_commits: Vec<_> = log_entries
                .iter()
                .filter(|e| commits.contains(&e.commit_id) && !e.empty)
                .collect();

            if non_empty_commits.len() <= 1 {
                println!("{}", crate::t!("messages.only_one_commit"));
                return Ok(());
            }

            // Get the first commit's change_id as stable target
            let first_entry = non_empty_commits.first().unwrap();
            let target_change_id = &first_entry.change_id;

            let full_message = if let Some(body_text) = body {
                format!("{}\n\n{}", message, body_text)
            } else {
                message
            };

            if dry_run {
                let count_str = non_empty_commits.len().to_string();
                println!(
                    "{}",
                    crate::t!(
                        "messages.pack_dry_run",
                        arg1 = count_str,
                        arg2 = target_change_id
                    )
                );
            } else {
                // Squash each subsequent commit into the first one
                for entry in non_empty_commits.iter().skip(1) {
                    jj_util::squash_into(&entry.commit_id, target_change_id, &full_message)
                        .with_context(|| {
                            format!(
                                "Failed to squash {} into {}",
                                entry.commit_id, target_change_id
                            )
                        })?;
                }
                // Update description of the combined commit
                jj_util::describe_change(target_change_id, &full_message)
                    .context(crate::t!("errors.config_read"))?;
                // Create a new working copy on top of the packed commit
                jj_util::new_commit(target_change_id).context(crate::t!("errors.config_read"))?;
                println!("{}", crate::t!("messages.pack_success", arg = full_message));
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
