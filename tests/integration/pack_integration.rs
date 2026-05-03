// Integration tests for pack command
// These tests require a real DeepSeek API key

use std::env;

fn get_api_key() -> Option<String> {
    env::var("DEEPSEEK_API_KEY").ok()
}

#[tokio::test]
#[ignore = "requires API key"]
async fn test_pack_basic() {
    // Test basic pack functionality
}
