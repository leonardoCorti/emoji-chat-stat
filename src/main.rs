use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::error::Error;
use csv::Writer;
use regex::Regex;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        eprintln!("Usage: {} <file_path>", args[0]);
        return Ok(());
    }
    let file_path = &args[1];

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut wtr = Writer::from_path(file_path.replace("txt", "csv"))?;

    wtr.write_record(&["Date", "Hour", "Name"])?;

    for line in reader.lines() {
        let line = line?;

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

        if rest.contains("💩") {
            match extract_time(rest){
                Some(new_time) => {
                    wtr.write_record(&[date, &new_time, name])?;
                },
                None => {
                    wtr.write_record(&[date, hour, name])?;
                } 
            }
        }


    }

    wtr.flush()?;
    println!("Data has been written to {}",file_path.replace("txt", "csv"));

    Ok(())
}

fn extract_time(s: &str) -> Option<String> {
    let re = Regex::new(r"([0-9]|[01]\d|2[0-3]):([0-5]\d)").unwrap();

    if let Some(caps) = re.captures(s) {
        return Some(caps[0].to_string());
    }
    
    None
}
