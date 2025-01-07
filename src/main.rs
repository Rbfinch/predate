use chrono::prelude::*;
use clap::{ArgAction, Parser}; // Correct the import for Parser and add ArgAction
use serde_json::json;
use serde_yaml::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::Write;
use std::process::Command as ProcessCommand;
use std::time::{Instant, SystemTime}; // Rename to avoid conflict with clap::Command

#[derive(Parser)]
#[command(
    name = "predate",
    author = "Nicholas D. Crosbie",
    version = clap::crate_version!(),
    about = "Catch bugs and performance regressions through automated system testing",
    long_about = "Copyright (c) 2024 Nicholas D. Crosbie, licensed under the MIT License."
)]
struct Cli {
    /// Write output to a JSON file
    #[arg(
        short = 'j',
        long = "json-out",
        action = ArgAction::SetTrue,
        help = "Write test output to a JSON file"
    )]
    json_out: bool,

    /// Control flag
    #[arg(
        short ='c',
        long = "control", 
        action = ArgAction::SetTrue,
        help = "Set the control ")]
    control: bool,

    /// Path to the tests YAML file
    #[arg(required = true, help = "Path to the tests YAML file")]
    path_to_tests_yaml: String,
}

fn main() {
    // Exit immediately if a command exits with a non-zero status
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        }
    });
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Print the operating system and CPU architecture
    let os_type = env::consts::OS;
    let cpu_arch = env::consts::ARCH;
    println!("OS: {}", os_type);
    println!("CPU: {}", cpu_arch);

    // Print the current date and time in ISO 8601 format
    let now: DateTime<Utc> = Utc::now();
    let now_str = now.to_rfc3339();
    println!("Current date and time: {}", now_str);

    // Check if the control flag is provided
    let grepq = if cli.control {
        "grepq"
    } else {
        "../target/release/grepq"
    };

    let test_file = &cli.path_to_tests_yaml;

    // Load tests and expected sizes from YAML file
    let yaml_content = fs::read_to_string(test_file)?;
    let yaml: Value = serde_yaml::from_str(&yaml_content)?;
    let tests_yaml = &yaml["tests"];
    let expected_sizes_yaml = &yaml["expected_sizes"];

    let mut tests = HashMap::new();
    let mut expected_sizes = HashMap::new();

    if let (Some(tests_yaml), Some(expected_sizes_yaml)) =
        (tests_yaml.as_mapping(), expected_sizes_yaml.as_mapping())
    {
        for (key, value) in tests_yaml {
            let key = key.as_str().unwrap();
            let value = value.as_str().unwrap().replace("$GREPQ", grepq);
            tests.insert(key.to_string(), value);
        }
        for (key, value) in expected_sizes_yaml {
            let key = key.as_str().unwrap();
            let value = value.as_i64().unwrap();
            expected_sizes.insert(key.to_string(), value);
        }
    }

    // Using an array to maintain the order of the tests
    let test_order = vec![
        "test-1", "test-2", "test-3", "test-4", "test-5", "test-6", "test-7", "test-8", "test-9",
        "test-10",
    ];

    // Color codes
    let bold = "\x1b[1m";
    let orange = "\x1b[38;2;255;165;0m";
    let reset = "\x1b[0m";

    println!("\nTests run:");
    println!(
        "{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
    );

    let mut json_output = vec![];

    let mut passing_tests = 0;
    let mut failing_tests = 0;

    for test in test_order {
        println!("{}{}{}", bold, test, reset);
        let command = &tests[test];
        println!("{}", command);

        let start_time = Instant::now();
        let output = if test == "test-7" || test == "test-8" {
            ProcessCommand::new("sh").arg("-c").arg(command).output()?
        } else if test == "test-10" {
            ProcessCommand::new("sh").arg("-c").arg(command).output()?;
            let actual_size = metadata("matches.json")?.len() as i64;
            if actual_size != expected_sizes[test] {
                println!("\n{}{} failed{}", orange, test, reset);
                println!(
                    "{}expected: {} bytes{}",
                    orange, expected_sizes[test], reset
                );
                println!("{}got: {} bytes{}", orange, actual_size, reset);
                println!("{}command was: {}{}", orange, command, reset);
                if cli.json_out {
                    json_output.push(json!({
                        "test": test,
                        "status": "failed",
                        "expected": expected_sizes[test],
                        "got": actual_size,
                        "command": command
                    }));
                }
                failing_tests += 1;
            } else {
                let duration = start_time.elapsed();
                let duration_str = format!("{:?}", duration);
                let (duration_value, duration_unit) = duration_str.split_at(
                    duration_str
                        .find(|c: char| !c.is_numeric() && c != '.')
                        .unwrap_or(duration_str.len()),
                );
                println!("Test {} completed in {:?}", test, duration);
                if cli.json_out {
                    json_output.push(json!({
                        "test": test,
                        "status": "passed",
                        "duration": duration_value.trim(),
                        "units": duration_unit.trim(),
                        "command": command
                    }));
                }
                passing_tests += 1;
            }
            continue;
        } else {
            ProcessCommand::new("sh")
                .arg("-c")
                .arg(format!("{} > /tmp/{}.txt", command, test))
                .output()?;
            let actual_size = metadata(format!("/tmp/{}.txt", test))?.len() as i64;
            if actual_size != expected_sizes[test] {
                println!("\n{}{} failed{}", orange, test, reset);
                println!(
                    "{}expected: {} bytes{}",
                    orange, expected_sizes[test], reset
                );
                println!("{}got: {} bytes{}", orange, actual_size, reset);
                println!(
                    "{}command was: {} > /tmp/{}.txt{}",
                    orange, command, test, reset
                );
                if cli.json_out {
                    json_output.push(json!({
                        "test": test,
                        "status": "failed",
                        "expected": expected_sizes[test],
                        "got": actual_size,
                        "command": format!("{} > /tmp/{}.txt", command, test)
                    }));
                }
                failing_tests += 1;
            } else {
                let duration = start_time.elapsed();
                let duration_str = format!("{:?}", duration);
                let (duration_value, duration_unit) = duration_str.split_at(
                    duration_str
                        .find(|c: char| !c.is_numeric() && c != '.')
                        .unwrap_or(duration_str.len()),
                );
                println!("Test {} completed in {:?}", test, duration);
                if cli.json_out {
                    json_output.push(json!({
                        "test": test,
                        "status": "passed",
                        "duration": duration_value.trim(),
                        "units": duration_unit.trim(),
                        "command": format!("{} > /tmp/{}.txt", command, test)
                    }));
                }
                passing_tests += 1;
            }
            continue;
        };

        let actual_count = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<i64>()?;
        if actual_count != expected_sizes[test] {
            println!("\n{}{} failed{}", orange, test, reset);
            println!(
                "{}expected: {} counts{}",
                orange, expected_sizes[test], reset
            );
            println!("{}got: {} counts{}", orange, actual_count, reset);
            println!("{}command was: {}{}", orange, command, reset);
            if cli.json_out {
                json_output.push(json!({
                    "test": test,
                    "status": "failed",
                    "expected": expected_sizes[test],
                    "got": actual_count,
                    "command": command
                }));
            }
            failing_tests += 1;
        } else {
            let duration = start_time.elapsed();
            let duration_str = format!("{:?}", duration);
            let (duration_value, duration_unit) = duration_str.split_at(
                duration_str
                    .find(|c: char| !c.is_numeric() && c != '.')
                    .unwrap_or(duration_str.len()),
            );
            println!("Test {} completed in {:?}", test, duration);
            if cli.json_out {
                json_output.push(json!({
                    "test": test,
                    "status": "passed",
                    "duration": duration_value.trim(),
                    "units": duration_unit.trim(),
                    "command": command
                }));
            }
            passing_tests += 1;
        }
    }

    println!("\nSummary:");
    println!("Passing tests: {}", passing_tests);
    println!("Failing tests: {}", failing_tests);

    if cli.json_out {
        let file_name = format!("{}.json", now_str); // Generate a default file name
        let mut file = File::create(file_name)?;
        file.write_all(serde_json::to_string_pretty(&json_output)?.as_bytes())?;
    } else {
        for entry in &json_output {
            println!("{}", serde_json::to_string_pretty(entry)?);
        }
    }

    Ok(())
}
