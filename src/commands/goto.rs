use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, GotoOutput, LlmOutput};

const SYSTEM_PROMPT: &str = r#"You are an expert at navigating version control history.

Given a user's request and the commit history, find the most appropriate commit to checkout.

You MUST respond in XML format with the following structure:

```xml
<goto>
  <summary>Brief description of why this commit was selected</summary>
  <target>The commit ID or revision to checkout</target>
</goto>
```

Rules:
- target should be a valid commit ID (full ID from the history) or jj revision alias (e.g., "@-", "@--")
- Use the full commit ID from the history for accuracy
- summary explains why this commit matches the user's request
- Be smart about interpreting user requests:
  - "initial commit" means the first commit in the history
  - "latest" or "most recent" means the most recent commit
  - "before X" means the commit just before the one that introduced X
  - "when X was added" means the commit that introduced X
- Always try to find a matching commit if the request makes sense
- If you cannot find a matching commit, respond with:

```xml
<reply>
  <message>Could not find a commit matching your request</message>
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

    let user_prompt = format!(
        "User request: {}\n\nCommit history:\n{}",
        query, log_text
    );

    let system_prompt = add_language_hint(SYSTEM_PROMPT, config);

    let response = client
        .chat(&system_prompt, &user_prompt, verbose, show_prompt, show_thinking, debug)
        .await
        .context("Failed to get LLM response")?;

    let output = parse_xml_output(&response).context("Failed to parse LLM XML response")?;

    match output {
        LlmOutput::Goto(GotoOutput { summary, target }) => {
            println!("{}", summary);
            if dry_run {
                println!("\n[Dry Run] Would checkout: {}", target);
            } else {
                jj_util::new_commit(&target).context("Failed to checkout target commit")?;
                println!("Checked out: {}", target);
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
