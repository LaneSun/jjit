use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug)]
pub enum LlmOutput {
    Commit(CommitOutput),
    Goto(GotoOutput),
    Pack(PackOutput),
    Reply(ReplyOutput),
}

#[derive(Debug)]
pub struct CommitOutput {
    pub summary: Option<String>,
    pub message: String,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct GotoOutput {
    pub summary: String,
    pub target: String,
}

#[derive(Debug)]
pub struct PackOutput {
    pub summary: String,
    pub commits: Vec<String>,
    pub message: String,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct ReplyOutput {
    pub message: String,
}

pub fn parse_xml_output(xml: &str) -> Result<LlmOutput> {
    // Extract root tag
    let root_tag = extract_root_tag(xml)?;

    match root_tag.as_str() {
        "commit" => parse_commit(xml),
        "goto" => parse_goto(xml),
        "pack" => parse_pack(xml),
        "reply" => parse_reply(xml),
        _ => {
            // Try to infer from content
            if xml.contains("<commit>") {
                parse_commit(xml)
            } else if xml.contains("<goto>") {
                parse_goto(xml)
            } else if xml.contains("<pack>") {
                parse_pack(xml)
            } else if xml.contains("<reply>") {
                parse_reply(xml)
            } else {
                Err(anyhow::anyhow!(
                    "Unknown or missing root tag in LLM response"
                ))
            }
        }
    }
}

fn extract_root_tag(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref())
                    .to_string()
                    .to_lowercase();
                return Ok(name);
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref())
                    .to_string()
                    .to_lowercase();
                return Ok(name);
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Err(anyhow::anyhow!("Could not find root tag in XML"))
}

fn parse_commit(xml: &str) -> Result<LlmOutput> {
    let fields = extract_fields(xml)?;

    let summary = fields.get("summary").cloned();
    let message = fields
        .get("message")
        .cloned()
        .context("Missing required field 'message' in commit response")?;
    let body = fields.get("body").cloned();

    Ok(LlmOutput::Commit(CommitOutput {
        summary,
        message,
        body,
    }))
}

fn parse_goto(xml: &str) -> Result<LlmOutput> {
    let fields = extract_fields(xml)?;

    let summary = fields
        .get("summary")
        .cloned()
        .context("Missing required field 'summary' in goto response")?;
    let target = fields
        .get("target")
        .cloned()
        .context("Missing required field 'target' in goto response")?;

    Ok(LlmOutput::Goto(GotoOutput { summary, target }))
}

fn parse_pack(xml: &str) -> Result<LlmOutput> {
    let fields = extract_fields(xml)?;

    let summary = fields
        .get("summary")
        .cloned()
        .context("Missing required field 'summary' in pack response")?;
    let commits = fields
        .get("commits")
        .cloned()
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let message = fields
        .get("message")
        .cloned()
        .context("Missing required field 'message' in pack response")?;
    let body = fields.get("body").cloned();

    Ok(LlmOutput::Pack(PackOutput {
        summary,
        commits,
        message,
        body,
    }))
}

fn parse_reply(xml: &str) -> Result<LlmOutput> {
    let fields = extract_fields(xml)?;

    let message = fields
        .get("message")
        .cloned()
        .context("Missing required field 'message' in reply response")?;

    Ok(LlmOutput::Reply(ReplyOutput { message }))
}

fn extract_fields(xml: &str) -> Result<std::collections::HashMap<String, String>> {
    let mut fields = std::collections::HashMap::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut tag_stack: Vec<String> = Vec::new();
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref())
                    .to_string()
                    .to_lowercase();
                tag_stack.push(name);
                current_text.clear();
            }
            Ok(Event::Text(e)) => {
                if !tag_stack.is_empty() {
                    let text = e.unescape().unwrap_or_default().to_string();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref())
                    .to_string()
                    .to_lowercase();
                if tag_stack.last() == Some(&name) {
                    // Only capture fields that are direct children of root (depth == 2)
                    if tag_stack.len() == 2 {
                        fields.insert(name, current_text.clone());
                    }
                    tag_stack.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                // If XML is malformed, try to extract what we can
                if fields.is_empty() {
                    return Err(anyhow::anyhow!("XML parse error: {}", e));
                }
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(fields)
}

pub fn format_summary(output: &LlmOutput) -> String {
    match output {
        LlmOutput::Commit(c) => c.summary.clone().unwrap_or_else(|| c.message.clone()),
        LlmOutput::Goto(g) => g.summary.clone(),
        LlmOutput::Pack(p) => p.summary.clone(),
        LlmOutput::Reply(r) => r.message.clone(),
    }
}
