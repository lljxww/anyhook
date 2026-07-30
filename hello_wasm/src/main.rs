use std::env;
use std::fs;

fn main() {
    let context_json = env::var("ANYHOOK_CONTEXT").unwrap_or_else(|_| "{}".to_string());
    let out_file = env::var("ANYHOOK_OUTPUT_FILE").unwrap_or_else(|_| "out.json".to_string());

    let output = format!(
        "{{\"status\": \"success\", \"message\": \"Hello from WASM!\", \"context\": {}}}",
        context_json
    );

    fs::write(out_file, output).unwrap();
}
