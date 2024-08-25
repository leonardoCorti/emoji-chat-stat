use csv::Reader;
use plotters::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use image::{DynamicImage, GenericImage, RgbaImage};
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

    let mut filenames_by_hour = Vec::new();
    // Creating histograms for each name
    for (name, hour_counts) in &data {
        create_histogram(name, hour_counts)?;
        filenames_by_hour.push(format!("{}-by-hour.png", name));
    }

    let one_image = args.contains(&"--one-image".to_string());
    if one_image {
        // Merge all generated images into a single image
        merge_images(&filenames_by_hour, "all-by-hour.png")?;
    }

    Ok(())
}

fn merge_images(image_paths: &[String], output_filename: &str) ->  Result<(), Box<dyn Error>> {
    let padding = 10;
    let images: Vec<DynamicImage> = image_paths.iter()
        .map(|path| image::open(path).expect("Failed to open image"))
        .collect();

    let image_width = images[0].width();
    let image_height = images[0].height();

    let total_width = (image_width + padding) * images.len() as u32 + padding;
    let total_height = image_height;

    let mut combined_image = RgbaImage::new(total_width, total_height);

    let mut x_offset = 0;

    for img in images {
        combined_image.copy_from(&img, x_offset, 0)?;
        x_offset += image_width + padding;
    }

    combined_image.save(output_filename)?;
    println!("Saved combined histogram to {}", output_filename);

    Ok(())
}

fn create_histogram(name: &str, hour_counts: &[u32; 24]) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}-by-hour.png", name);
    let root = BitMapBackend::new(&filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = *hour_counts.iter().max().unwrap_or(&0);

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("distribuzione oraria di {}", name), ("sans-serif", 40))
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

