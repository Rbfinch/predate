#![allow(unused)]
fn main() {
    use std::process::Command;

    let mut echo_hello = Command::new("bash");
    echo_hello.arg("-c").arg("echo hello");
    let hello_1 = echo_hello.output().expect("failed to execute process");
    assert!(hello_1.status.success(), "First command failed");
    println!("First command executed successfully");
    let hello_2 = echo_hello.output().expect("failed to execute process");
    assert!(hello_2.status.success(), "Second command failed");
    println!("Second command executed successfully");
}
