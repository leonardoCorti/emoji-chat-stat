use std::process::{Command, exit};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: emoji-chat-stat  <input.txt>");
        exit(1);
    }

    let input_file = &args[1];
    let output_file = input_file.replace(".txt", ".csv");

    let status = Command::new("emoji2csv")
        .arg(input_file)
        .status()
        .expect("Failed to execute emoji2csv");

    if !status.success() {
        eprintln!("emoji2csv failed");
        exit(1);
    }

    let status = Command::new("emojicsv2graph")
        .arg(output_file)
        .arg("--one-image")
        .status()
        .expect("Failed to execute emojicsv2graph");

    if !status.success() {
        eprintln!("emojicsv2graph failed");
        exit(1);
    }
}

