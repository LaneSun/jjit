// Integration tests for commit command
// These tests require a real DeepSeek API key

use std::env;

fn get_api_key() -> Option<String> {
    env::var("DEEPSEEK_API_KEY").ok()
}

#[tokio::test]
#[ignore = "requires API key"]
async fn test_commit_no_changes() {
    // This test should be run in an empty jj repo
    // It should handle the case where there are no changes
}
