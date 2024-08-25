use std::process::{Command, exit};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: bin0 <input.txt>");
        exit(1);
    }

    let input_file = &args[1];
    let output_file = input_file.replace(".txt", ".csv");

    let status = Command::new("cacca_text.exe")
        .arg(input_file)
        .status()
        .expect("Failed to execute cacca_text");

    if !status.success() {
        eprintln!("cacca_text failed");
        exit(1);
    }

    let status = Command::new("cacca_graph.exe")
        .arg(output_file)
        .status()
        .expect("Failed to execute cacca_graph");

    if !status.success() {
        eprintln!("cacca_graph failed");
        exit(1);
    }
}

