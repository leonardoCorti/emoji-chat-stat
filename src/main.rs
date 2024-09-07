use std::process::{Command, exit};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: emoji-chat-stat  <input.txt> <emoji_searched>");
        exit(1);
    }

    let input_file = &args[1];
    let emoji_searched = &args[2];
    let output_file = input_file.replace(".txt", ".csv");

    let status = Command::new("../../target/release/emoji2csv")
        .arg(emoji_searched)
        .arg(input_file)
        .arg("-o").arg(output_file.clone())
        .status()
        .expect("Failed to execute emoji2csv");

    if !status.success() {
        eprintln!("emoji2csv failed");
        exit(1);
    }

    let status = Command::new("emojicsv2graph")
        .arg(output_file)
        .arg("--one-image")
        .arg("--clean")
        .status()
        .expect("Failed to execute emojicsv2graph");

    if !status.success() {
        eprintln!("emojicsv2graph failed");
        exit(1);
    }
}

