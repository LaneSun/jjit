use jjit::output::parse_xml_output;

#[test]
fn test_parse_commit_xml() {
    let xml = r#"
    <commit>
      <summary>检测到 3 个文件变更</summary>
      <message>feat(auth): 添加用户登录验证模块</message>
      <body>新增用户名密码验证逻辑</body>
    </commit>
    "#;

    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_goto_xml() {
    let xml = r#"
    <goto>
      <summary>找到目标提交</summary>
      <target>abc123def456</target>
    </goto>
    "#;

    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_pack_xml() {
    let xml = r#"
    <pack>
      <summary>组合提交</summary>
      <range>abc123:def456</range>
      <message>feat: 实现功能</message>
    </pack>
    "#;

    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_reply_xml() {
    let xml = r#"
    <reply>
      <message>没有变更</message>
    </reply>
    "#;

    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_malformed_xml() {
    let xml = r#"<commit><summary>test</summary><message>feat: test</message>"#;
    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_with_extra_text() {
    let xml = r#"
    Here is the result:
    <commit>
      <summary>test</summary>
      <message>feat: test</message>
    </commit>
    Some extra text.
    "#;
    let result = parse_xml_output(xml);
    assert!(result.is_ok());
}
