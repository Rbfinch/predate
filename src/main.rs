use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::process;

fn main() {
    let file_content = match fs::read_to_string("test_data.json") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Unable to read file: {}", err);
            process::exit(1);
        }
    };

    let json_data: Value = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("JSON was not well-formatted: {}", err);
            process::exit(1);
        }
    };

    let predicate = predicate::function(|json: &str| {
        let parsed_json: Value = serde_json::from_str(json).unwrap();
        parsed_json == json_data
    });

    let mut cmd = match Command::cargo_bin("your_cli_app") {
        Ok(command) => command,
        Err(err) => {
            eprintln!("Binary not found: {}", err);
            process::exit(1);
        }
    };

    cmd.assert().success().stdout(predicate);
}
