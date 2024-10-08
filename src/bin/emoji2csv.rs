use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::error::Error;
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let emoji_searched = if !args[1].starts_with('-') {
        &args[1]
    } else {
        eprintln!("Usage: {} <emoji_to_search> [<input_file>] [-o <output_file>] [-i]", args[0]);
        return Ok(());
    };

    let input: Box<dyn BufRead> = if args.len() > 2 && !args[2].starts_with('-') {
        Box::new(BufReader::new(File::open(&args[2])?))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let output: Box<dyn Write> = if let Some(pos) = args.iter().position(|x| x == "-o") {
        if pos + 1 < args.len() {
            Box::new(File::create(&args[pos + 1])?)
        } else {
            eprintln!("Expected output file after -o");
            return Ok(());
        }
    } else {
        Box::new(io::stdout())
    };

    let is_case_insensitive = args.contains(&"-i".to_string());
    let string_searched = match is_case_insensitive {
        true => emoji_searched.to_lowercase(),
        false => emoji_searched.to_string(),
    };

    process_input(input, output, &string_searched, is_case_insensitive)?;
    Ok(())
 }


fn process_input<R: BufRead, W: Write>(
    reader: R,
    mut writer: W,
    emoji_searched: &str,
    is_case_insensitive: bool
) -> io::Result<()> {
    writer.write_all(b"Date,Hour,Name\n")?;

    for line in reader.lines() {
        let mut line = line?;
        if line.starts_with("[") { // iphone parsing
            let new_line = line.strip_prefix("[").unwrap();
            let new_line = new_line.replacen("]", " -", 1);
            line = new_line;
        }

        let (date, rest) = match line.split_once(", ") {
            Some(split) => split,
            None => continue,
        };

        let (hour, rest) = match rest.split_once(" - ") {
            Some(split) => split,
            None => continue,
        };

        let (name, rest) = match rest.split_once(": ") {
            Some(split) => split,
            None => continue,
        };

        let last_section = match is_case_insensitive{
            true => rest.to_lowercase(),
            false => rest.to_string(),
        };

        if last_section.contains(emoji_searched) {
            let new_time = extract_time(rest).unwrap_or_else(|| hour.to_string());
            writer.write_all(format!("{},{},{}\n", date, new_time, name).as_bytes())?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn extract_time(s: &str) -> Option<String> {
    let re = Regex::new(r"([0-9]|[01]\d|2[0-3]):([0-5]\d)").unwrap();

    if let Some(caps) = re.captures(s) {
        return Some(caps[0].to_string());
    }
    
    None
}
