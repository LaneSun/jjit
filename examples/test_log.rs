fn main() {
    let path = std::env::current_dir().unwrap();
    let output = vcs_runner::run_jj_utf8(
        &path,
        &["log", "-r", "all()", "--template", vcs_runner::LOG_TEMPLATE],
    )
    .unwrap();

    println!("Raw output:");
    println!("{}", output);
    println!("\n---");

    let result = vcs_runner::parse_log_output(&output);
    println!(
        "Parsed {} entries, skipped {} lines",
        result.entries.len(),
        result.skipped.len()
    );

    if !result.skipped.is_empty() {
        println!("\nSkipped lines:");
        for line in &result.skipped {
            println!("  {}", line);
        }
    }
}
