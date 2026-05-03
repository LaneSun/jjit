use std::collections::HashMap;

#[test]
fn test_config_default() {
    let config = jjit::config::Config::default();
    assert_eq!(config.get("model"), Some("deepseek-v4-flash".to_string()));
    assert_eq!(
        config.get("base_url"),
        Some("https://api.deepseek.com".to_string())
    );
}
