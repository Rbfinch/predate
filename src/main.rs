use chrono::prelude::*;
use clap::{Arg, Command}; // Correct the import for Command
use serde_json::json;
use serde_yaml::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command as ProcessCommand;
use std::time::{Instant, SystemTime}; // Rename to avoid conflict with clap::Command

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
    // Parse command line arguments
    let matches = Command::new("predate")
        .arg(
            Arg::new("json-out")
                .short('j')
                .long("json-out")
                .help("Write output to a JSON file")
                .action(clap::ArgAction::SetTrue), // Correctly indicate no value needed for this flag
        )
        .arg(Arg::new("control").help("Control flag").index(1))
        .arg(
            Arg::new("path_to_tests_yaml")
                .help("Path to the tests YAML file")
                .index(2),
        )
        .get_matches();

    // Print the operating system and CPU architecture
    let os_type = env::consts::OS;
    let cpu_arch = env::consts::ARCH;
    println!("OS: {}", os_type);
    println!("CPU: {}", cpu_arch);

    // Print the current date and time in ISO 8601 format
    let now: DateTime<Utc> = Utc::now();
    let now_str = now.to_rfc3339();
    println!("Current date and time: {}", now_str);

    let stat_cmd = if os_type == "linux" {
        "stat -c %s"
    } else if os_type == "macos" {
        "stat -f %z"
    } else {
        return Err("OS: Unknown".into());
    };

    // Check if the control flag is provided
    let args: Vec<String> = env::args().collect();
    let (grepq, test_file) = if matches.contains_id("control") {
        if args.len() < 3 {
            return Err("Usage: <program> [control] <path_to_tests_yaml>".into());
        }
        ("grepq", &args[2])
    } else {
        if args.len() < 2 {
            return Err("Usage: <program> [control] <path_to_tests_yaml>".into());
        }
        ("../target/release/grepq", &args[1])
    };

    // Check if the path to the test file is provided
    if test_file.is_empty() {
        return Err("Usage: <program> [control] <path_to_tests_yaml>".into());
    }

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

    for test in test_order {
        println!("{}{}{}", bold, test, reset);
        let command = &tests[test];
        println!("{}", command);

        let start_time = Instant::now();
        let output = if test == "test-7" || test == "test-8" {
            ProcessCommand::new("sh").arg("-c").arg(command).output()?
        } else if test == "test-10" {
            ProcessCommand::new("sh").arg("-c").arg(command).output()?;
            let actual_size = ProcessCommand::new("sh")
                .arg("-c")
                .arg(format!("{} matches.json", stat_cmd))
                .output()?;
            let actual_size = String::from_utf8_lossy(&actual_size.stdout)
                .trim()
                .parse::<i64>()?;
            if actual_size != expected_sizes[test] {
                println!("\n{}{} failed{}", orange, test, reset);
                println!(
                    "{}expected: {} bytes{}",
                    orange, expected_sizes[test], reset
                );
                println!("{}got: {} bytes{}", orange, actual_size, reset);
                println!("{}command was: {}{}", orange, command, reset);
                if matches.contains_id("json-out") {
                    json_output.push(json!({
                        "test": test,
                        "status": "failed",
                        "expected": expected_sizes[test],
                        "got": actual_size,
                        "command": command
                    }));
                }
            } else {
                let duration = start_time.elapsed();
                let duration_str = format!("{:?}", duration);
                let (duration_value, duration_unit) = duration_str.split_at(
                    duration_str
                        .find(|c: char| !c.is_numeric() && c != '.')
                        .unwrap_or(duration_str.len()),
                );
                println!("Test {} completed in {:?}", test, duration);
                if matches.contains_id("json-out") {
                    json_output.push(json!({
                        "test": test,
                        "status": "passed",
                        "duration": duration_value.trim(),
                        "units": duration_unit.trim()
                    }));
                }
            }
            continue;
        } else {
            ProcessCommand::new("sh")
                .arg("-c")
                .arg(format!("{} > /tmp/{}.txt", command, test))
                .output()?;
            let actual_size = ProcessCommand::new("sh")
                .arg("-c")
                .arg(format!("{} /tmp/{}.txt", stat_cmd, test))
                .output()?;
            let actual_size = String::from_utf8_lossy(&actual_size.stdout)
                .trim()
                .parse::<i64>()?;
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
                if matches.contains_id("json-out") {
                    json_output.push(json!({
                        "test": test,
                        "status": "failed",
                        "expected": expected_sizes[test],
                        "got": actual_size,
                        "command": format!("{} > /tmp/{}.txt", command, test)
                    }));
                }
            } else {
                let duration = start_time.elapsed();
                let duration_str = format!("{:?}", duration);
                let (duration_value, duration_unit) = duration_str.split_at(
                    duration_str
                        .find(|c: char| !c.is_numeric() && c != '.')
                        .unwrap_or(duration_str.len()),
                );
                println!("Test {} completed in {:?}", test, duration);
                if matches.contains_id("json-out") {
                    json_output.push(json!({
                        "test": test,
                        "status": "passed",
                        "duration": duration_value.trim(),
                        "units": duration_unit.trim()
                    }));
                }
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
            if matches.contains_id("json-out") {
                json_output.push(json!({
                    "test": test,
                    "status": "failed",
                    "expected": expected_sizes[test],
                    "got": actual_count,
                    "command": command
                }));
            }
        } else {
            let duration = start_time.elapsed();
            let duration_str = format!("{:?}", duration);
            let (duration_value, duration_unit) = duration_str.split_at(
                duration_str
                    .find(|c: char| !c.is_numeric() && c != '.')
                    .unwrap_or(duration_str.len()),
            );
            println!("Test {} completed in {:?}", test, duration);
            if matches.contains_id("json-out") {
                json_output.push(json!({
                    "test": test,
                    "status": "passed",
                    "duration": duration_value.trim(),
                    "units": duration_unit.trim()
                }));
            }
        }
    }

    if matches.contains_id("json-out") {
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
