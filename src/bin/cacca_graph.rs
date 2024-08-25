use csv::Reader;
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the first command-line argument as the file path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        return Ok(());
    }
    let file_path = &args[1];
    
    // Reading and parsing the CSV file
    let mut reader = Reader::from_path(file_path)?;

    let mut data: HashMap<String, [u32; 24]> = HashMap::new();

    for result in reader.records() {
        let record = result?;
        let name = record[2].to_string();
        let hour: usize = record[1].split(':').next().unwrap().parse()?;

        // Increment the hour count for the name
        data.entry(name.clone())
            .or_insert([0; 24])[hour] += 1;
    }

    // Creating histograms for each name
    for (name, hour_counts) in &data {
        create_histogram(name, hour_counts)?;
    }

    Ok(())
}

fn create_histogram(name: &str, hour_counts: &[u32; 24]) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}.png", name);
    let root = BitMapBackend::new(&filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = *hour_counts.iter().max().unwrap_or(&0);

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Hourly Distribution for {}", name), ("sans-serif", 40))
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(0..23, 0u32..max_count)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data((0..24).zip(hour_counts.iter().copied())),
    )?;

    root.present()?;
    println!("Saved histogram to {}", filename);

    Ok(())
}

