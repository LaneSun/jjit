use anyhow::{Context, Result};

use crate::config::Config;
use crate::jj_util;
use crate::llm::LlmClient;
use crate::output::{parse_xml_output, CommitOutput, LlmOutput};

const SYSTEM_PROMPT: &str = r#"You are an expert at writing commit messages following the Conventional Commits specification.

Analyze the provided diff and generate an appropriate commit message.

You MUST respond in XML format with the following structure:

```xml
<commit>
  <message>The commit message title (follow Conventional Commits)</message>
  <body>Optional detailed description (can be multi-line)</body>
</commit>
```

Rules:
- message should follow Conventional Commits format (e.g., "feat(scope): description")
- body is optional, use it for breaking changes or detailed explanations
- If there are no changes to commit, respond with:

```xml
<reply>
  <message>No changes detected in the working copy</message>
</reply>
```
"#;

pub async fn run(config: &Config, dry_run: bool) -> Result<()> {
    let verbose = config.get("verbose") == Some("true".to_string());
    let show_prompt = config.get("show_prompt") == Some("true".to_string());
    let show_thinking = config.get("show_thinking") != Some("false".to_string());
    let debug = config.get("debug") == Some("true".to_string());

    // Check if there are changes
    if !jj_util::has_changes().context("Failed to check working copy status")? {
        println!("No changes detected in the working copy");
        return Ok(());
    }

    // Get diff information
    let diff_summary = jj_util::get_diff_summary().context("Failed to get diff summary")?;
    let diff = jj_util::get_diff().context("Failed to get diff")?;

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
        LlmOutput::Commit(CommitOutput { message, body, .. }) => {
            let full_message = if let Some(body_text) = body {
                format!("{}\n\n{}", message, body_text)
            } else {
                message
            };

            if dry_run {
                println!("[dry-run] {}", full_message);
            } else {
                jj_util::commit(&full_message).context("Failed to create commit")?;
                println!("[commit] {}", full_message);
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
