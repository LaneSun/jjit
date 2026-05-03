use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::path::Path;
use vcs_runner::{parse_diff_summary, run_jj_utf8, FileChange};

fn repo_path() -> std::path::PathBuf {
    env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
}

fn map_run_error(e: vcs_runner::RunError) -> anyhow::Error {
    anyhow::anyhow!("jj command failed: {}", e)
}

fn run_jj_with_debug(args: &[&str]) -> Result<String> {
    let path = repo_path();
    if std::env::var("JJIT_DEBUG").is_ok() || std::env::var("DEBUG").is_ok() {
        eprintln!("[DEBUG] jj {}", args.join(" "));
    }
    run_jj_utf8(&path, args).map_err(map_run_error)
}

// Custom template with full commit_id for reliable revset resolution
// Note: commit_id and change_id are not string types, need stringify() first
const FULL_LOG_TEMPLATE: &str = concat!(
    r#"'{"commitId":' ++ stringify(commit_id).escape_json()"#,
    r#" ++ ',"changeId":' ++ stringify(change_id).escape_json()"#,
    r#" ++ ',"authorName":' ++ author.name().escape_json()"#,
    r#" ++ ',"authorEmail":' ++ stringify(author.email()).escape_json()"#,
    r#" ++ ',"authorDate":' ++ author.timestamp().format("%Y-%m-%d %H:%M:%S").escape_json()"#,
    r#" ++ ',"description":' ++ description.escape_json()"#,
    r#" ++ ',"parents":[' ++ parents.map(|p| stringify(p.commit_id()).escape_json()).join(',') ++ ']'"#,
    r#" ++ ',"localBookmarks":[' ++ local_bookmarks.map(|b| b.name().escape_json()).join(',') ++ ']'"#,
    r#" ++ ',"remoteBookmarks":[' ++ remote_bookmarks.map(|b| stringify(b.name() ++ "@" ++ b.remote()).escape_json()).join(',') ++ ']'"#,
    r#" ++ ',"isWorkingCopy":' ++ if(current_working_copy, '"true"', '"false"')"#,
    r#" ++ ',"conflict":' ++ if(conflict, '"true"', '"false"')"#,
    r#" ++ ',"empty":' ++ if(empty, '"true"', '"false"')"#,
    r#" ++ '}' ++ "\n""#,
);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLogEntry {
    commit_id: String,
    change_id: String,
    author_name: String,
    author_email: String,
    author_date: String,
    description: String,
    parents: Vec<String>,
    local_bookmarks: Vec<String>,
    remote_bookmarks: Vec<String>,
    is_working_copy: String,
    conflict: String,
    empty: String,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub commit_id: String,
    pub change_id: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub description: String,
    pub parents: Vec<String>,
    pub local_bookmarks: Vec<String>,
    pub remote_bookmarks: Vec<String>,
    pub is_working_copy: bool,
    pub conflict: bool,
    pub empty: bool,
}

impl LogEntry {
    pub fn summary(&self) -> &str {
        self.description.lines().next().unwrap_or("")
    }
}

fn parse_log_entries(output: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let raw: RawLogEntry = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };
        entries.push(LogEntry {
            commit_id: raw.commit_id,
            change_id: raw.change_id,
            author_name: raw.author_name,
            author_email: raw.author_email,
            author_date: raw.author_date,
            description: raw.description,
            parents: raw.parents.into_iter().filter(|p| !p.is_empty()).collect(),
            local_bookmarks: raw
                .local_bookmarks
                .into_iter()
                .filter(|b| !b.is_empty())
                .collect(),
            remote_bookmarks: raw
                .remote_bookmarks
                .into_iter()
                .filter(|b| !b.is_empty())
                .collect(),
            is_working_copy: raw.is_working_copy == "true",
            conflict: raw.conflict == "true",
            empty: raw.empty == "true",
        });
    }
    entries
}

pub fn get_repo_root() -> Result<String> {
    let output = run_jj_with_debug(&["root"]).context("Failed to get jj repo root")?;
    Ok(output.trim().to_string())
}

pub fn get_status() -> Result<String> {
    run_jj_with_debug(&["status"]).context("Failed to get jj status")
}

pub fn get_diff_summary() -> Result<Vec<FileChange>> {
    let output = run_jj_with_debug(&["diff", "--summary"]).context("Failed to get diff summary")?;
    let changes = parse_diff_summary(&output);
    Ok(changes)
}

pub fn get_diff() -> Result<String> {
    run_jj_with_debug(&["diff"]).context("Failed to get diff")
}

pub fn get_diff_for_rev(rev: &str) -> Result<String> {
    run_jj_with_debug(&["diff", "-r", rev]).context("Failed to get diff for revision")
}

pub fn get_log_all() -> Result<Vec<LogEntry>> {
    let output = run_jj_with_debug(&[
        "log",
        "-r",
        "all()",
        "--template",
        FULL_LOG_TEMPLATE,
        "--no-graph",
    ])
    .context("Failed to get jj log")?;
    Ok(parse_log_entries(&output))
}

pub fn get_log_today() -> Result<Vec<LogEntry>> {
    let output = run_jj_with_debug(&[
        "log",
        "-r",
        "author_date(after:'today 00:00')",
        "--template",
        FULL_LOG_TEMPLATE,
        "--no-graph",
    ])
    .context("Failed to get today's jj log")?;
    Ok(parse_log_entries(&output))
}

pub fn commit(message: &str) -> Result<()> {
    run_jj_with_debug(&["commit", "-m", message]).context("Failed to create commit")?;
    Ok(())
}

pub fn new_commit(rev: &str) -> Result<()> {
    run_jj_with_debug(&["new", rev]).context("Failed to create new commit")?;
    Ok(())
}

pub fn squash(range: &str, message: &str) -> Result<()> {
    run_jj_with_debug(&["squash", "-r", range, "-m", message])
        .context("Failed to squash commits")?;
    Ok(())
}

pub fn squash_working_copy(message: &str) -> Result<()> {
    run_jj_with_debug(&["squash", "-m", message])
        .context("Failed to squash working copy commit")?;
    Ok(())
}

pub fn squash_from_into(from: &str, into: &str, message: &str) -> Result<()> {
    run_jj_with_debug(&["squash", "--from", from, "--into", into, "-m", message])
        .context("Failed to squash commits")?;
    Ok(())
}

pub fn describe_commit(commit_id: &str, message: &str) -> Result<()> {
    run_jj_with_debug(&["describe", "-r", commit_id, "-m", message])
        .context("Failed to describe commit")?;
    Ok(())
}

pub fn describe_change(change_id: &str, message: &str) -> Result<()> {
    run_jj_with_debug(&["describe", "-r", change_id, "-m", message])
        .context("Failed to describe commit by change id")?;
    Ok(())
}

pub fn squash_into(from: &str, into: &str, message: &str) -> Result<()> {
    run_jj_with_debug(&["squash", "--from", from, "--into", into, "-m", message])
        .context("Failed to squash commit")?;
    Ok(())
}

pub fn get_parent_commit(commit_id: &str) -> Result<String> {
    // Use commit_id- to get the parent, with a template that outputs just the commit ID
    let template = r#"stringify(commit_id) ++ "\n""#;
    let output = run_jj_with_debug(&[
        "log",
        "-r",
        &format!("{}-", commit_id),
        "--template",
        template,
        "--no-graph",
    ])
    .context("Failed to get parent commit")?;
    Ok(output.trim().to_string())
}

pub fn has_changes() -> Result<bool> {
    let summary = get_diff_summary()?;
    Ok(!summary.is_empty())
}
