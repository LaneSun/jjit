use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, LlmOutput, PackOutput};

const SYSTEM_PROMPT: &str = r#"You are an expert at organizing version control history.

Given a user's request and the commit history, determine which commits should be combined (squashed) together.

You MUST respond in XML format with the following structure:

```xml
<pack>
  <summary>Brief description of what commits are being combined and why</summary>
  <commits>commit_id1,commit_id2,commit_id3</commits>
  <message>The new commit message for the combined commit</message>
  <body>Optional detailed description</body>
</pack>
```

Rules:
- commits is a comma-separated list of full commit IDs to squash together (from oldest to newest)
- message should follow Conventional Commits format
- summary explains which commits are being combined
- Be smart about interpreting user requests:
  - "all commits" or "everything" means combine all non-root, non-empty commits
  - "today's commits" means commits made today (use the Date field)
  - "last N commits" means the N most recent non-empty commits
  - "related to X" means commits whose descriptions mention X
- Do NOT include empty commits (like the working copy) in the commits list
- Important: If there is only one non-empty, non-root commit, do NOT try to pack it. Respond with:
  ```xml
  <reply>
    <message>Only one commit found, nothing to pack</message>
  </reply>
  ```
- Always try to find matching commits if the request makes sense
- If there are no commits to pack, respond with:

```xml
<reply>
  <message>No matching commits found to pack</message>
</reply>
```
"#;

pub async fn run(config: &Config, query: &str, dry_run: bool) -> Result<()> {
    let verbose = config.get("verbose") == Some("true".to_string());
    let show_prompt = config.get("show_prompt") == Some("true".to_string());
    let show_thinking = config.get("show_thinking") != Some("false".to_string());
    let debug = config.get("debug") == Some("true".to_string());

    // Get commit history
    let log_entries = jj_util::get_log_all().context("Failed to get commit history")?;

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

    let system_prompt = add_language_hint(SYSTEM_PROMPT, config);

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
        .context("Failed to get LLM response")?;

    let output = parse_xml_output(&response).context("Failed to parse LLM XML response")?;

    match output {
        LlmOutput::Pack(PackOutput {
            summary,
            commits,
            message,
            body,
        }) => {
            println!("{}", summary);

            if commits.is_empty() {
                return Err(anyhow::anyhow!("LLM returned empty commits list for pack"));
            }

            // Filter out empty commits (like working copy)
            let non_empty_commits: Vec<_> = log_entries
                .iter()
                .filter(|e| commits.contains(&e.commit_id) && !e.empty)
                .collect();

            if non_empty_commits.len() <= 1 {
                println!("Only one non-empty commit found, nothing to pack");
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
                println!(
                    "[dry-run] pack {} commits into {}",
                    non_empty_commits.len(),
                    target_change_id
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
                    .context("Failed to describe combined commit")?;
                // Create a new working copy on top of the packed commit
                jj_util::new_commit(target_change_id)
                    .context("Failed to create new working copy after pack")?;
                println!("[pack] {}", full_message);
            }
        }
        LlmOutput::Reply(reply) => {
            println!("{}", reply.message);
        }
        _ => {
            return Err(anyhow::anyhow!("Unexpected response type from LLM"));
        }
    }

    Ok(())
}

fn add_language_hint(prompt: &str, config: &Config) -> String {
    match config.get("language") {
        Some(lang) if !lang.is_empty() && lang != "en" => {
            format!("{}\n\nYou MUST respond in {} language.", prompt, lang)
        }
        _ => prompt.to_string(),
    }
}
