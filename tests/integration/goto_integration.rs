// Integration tests for goto command
// These tests require a real DeepSeek API key

use std::env;

fn get_api_key() -> Option<String> {
    env::var("DEEPSEEK_API_KEY").ok()
}

#[tokio::test]
#[ignore = "requires API key"]
async fn test_goto_basic() {
    // Test basic goto functionality
}
