use chrono::Datelike;
use chrono::{NaiveDate, Weekday};
use csv::Reader;
use image::{DynamicImage, GenericImage, RgbaImage};
use plotters::prelude::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::{collections::HashMap, io::stdin};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-h".to_string()) {
        eprintln!(
            "Usage: {} [file_path] [--one-image] [--clean] [--new-stats]",
            args[0]
        );
        return Ok(());
    }

    let reader: Reader<Box<dyn std::io::Read>>;
    if args.len() > 1 {
        if args[1].starts_with("--") {
            reader = Reader::from_reader(Box::new(stdin()));
        } else {
            reader = Reader::from_reader(Box::new(File::open(&args[1])?));
        }
    } else {
        reader = Reader::from_reader(Box::new(stdin()));
    }

    let (data_hours, data_day, records) = process_input(reader)?;

    let one_image = args.contains(&"--one-image".to_string());
    let clean = args.contains(&"--clean".to_string());
    let new_stats = args.contains(&"--new-stats".to_string());

    let (max_hour_count, max_day_count) = find_max(one_image, &data_day, &data_hours);

    let mut filenames_by_weekday = Vec::new();
    let mut filenames_by_hour = Vec::new();

    for (name, day_count) in &data_day {
        create_histogram_weekday(name, day_count, max_day_count)?;
        filenames_by_weekday.push(format!("{}-by-weekday.png", name));
    }

    for (name, hour_counts) in &data_hours {
        create_histogram_hours(name, hour_counts, max_hour_count)?;
        filenames_by_hour.push(format!("{}-by-hour.png", name));
    }

    if one_image {
        filenames_by_hour.sort();
        filenames_by_weekday.sort();
        merge_horizontal_images(&filenames_by_hour, "all-by-hour.png")?;
        merge_horizontal_images(&filenames_by_weekday, "all-by-weekday.png")?;
        let images: Vec<&str> = vec!["all-by-hour.png", "all-by-weekday.png"];
        merge_vertical_images(&images, "all-graphs.png")?;
    }

    if clean {
        filenames_by_weekday
            .iter()
            .for_each(|f| std::fs::remove_file(f).unwrap());
        filenames_by_hour
            .iter()
            .for_each(|f| std::fs::remove_file(f).unwrap());
        std::fs::remove_file("all-by-hour.png")?;
        std::fs::remove_file("all-by-weekday.png")?;
    }

    if new_stats {
        let weekly = compute_winners(&records, |date| {
            let w = date.iso_week();
            format!("{}-W{:02}", w.year(), w.week())
        });
        let monthly = compute_winners(&records, |date| {
            format!("{}-{:02}", date.year(), date.month())
        });
        let yearly = compute_winners(&records, |date| {
            format!("{}", date.year())
        });

        render_winner_chart("weekly-top.png", "Vincitore settimanale", &weekly)?;
        render_winner_chart("monthly-top.png", "Vincitore mensile", &monthly)?;
        render_winner_chart("yearly-top.png", "Vincitore annuale", &yearly)?;

        println!("Saved new-stats images (weekly-top.png, monthly-top.png, yearly-top.png)");
    }

    Ok(())
}

fn find_max(
    one_image: bool,
    data_day: &HashMap<String, HashMap<Weekday, u32>>,
    data_hours: &HashMap<String, [u32; 24]>,
) -> (Option<u32>, Option<u32>) {
    let mut max_hour_count: Option<u32> = None;
    let mut max_day_count: Option<u32> = None;
    if one_image {
        let mut max_hour: u32 = 0;
        let mut max_day: u32 = 0;
        for (_, day_count) in data_day {
            let weekdays = vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ];
            let counts: Vec<u32> = weekdays
                .iter()
                .map(|&d| day_count.get(&d).cloned().unwrap_or(0))
                .collect();
            let max_day_tmp = *counts.iter().max().unwrap_or(&0);
            if max_day_tmp > max_day {
                max_day = max_day_tmp;
            }
        }

        for (_, hour_counts) in data_hours {
            let max_hour_tmp = *hour_counts.iter().max().unwrap_or(&0);
            if max_hour_tmp > max_hour {
                max_hour = max_hour_tmp;
            }
        }
        max_hour_count = Some(max_hour);
        max_day_count = Some(max_day);
    }
    (max_hour_count, max_day_count)
}

fn process_input(
    mut reader: Reader<Box<dyn Read>>,
) -> Result<
    (
        HashMap<String, [u32; 24]>,
        HashMap<String, HashMap<Weekday, u32>>,
        Vec<(NaiveDate, String)>,
    ),
    Box<dyn Error>,
> {
    let mut data: HashMap<String, [u32; 24]> = HashMap::new();
    let mut data_day: HashMap<String, HashMap<Weekday, u32>> = HashMap::new();
    let mut records = Vec::new();
    for result in reader.records() {
        let record = result?;
        let name = record[2].to_string();
        let hour: usize = record[1].split(':').next().unwrap().parse()?;

        data.entry(name.clone()).or_insert([0; 24])[hour] += 1;

        let date_str = &record[0];
        let date = NaiveDate::parse_from_str(date_str, "%d/%m/%y")?;
        let name = record[2].to_string();
        let weekday = date.weekday();
        *data_day
            .entry(name.clone())
            .or_insert(HashMap::new())
            .entry(weekday)
            .or_insert(0) += 1;

        records.push((date, name));
    }
    Ok((data, data_day, records))
}

fn merge_horizontal_images(
    image_paths: &[String],
    output_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let padding = 10;
    let images: Vec<DynamicImage> = image_paths
        .iter()
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

fn create_histogram_weekday(
    name: &str,
    day_count: &HashMap<Weekday, u32>,
    max: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}-by-weekday.png", name);
    let root = BitMapBackend::new(&filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let weekdays = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let counts: Vec<u32> = weekdays
        .iter()
        .map(|&d| day_count.get(&d).cloned().unwrap_or(0))
        .collect();
    let max_count = max.unwrap_or(*counts.iter().max().unwrap_or(&0));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("distribuzione settimanale {}", name),
            ("sans-serif", 40),
        )
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(0..7, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_desc("giorno della settimana")
        .y_desc("conta")
        .x_labels(7)
        .x_label_formatter(&|x| match x {
            0 => "Mon".to_string(),
            1 => "Tue".to_string(),
            2 => "Wed".to_string(),
            3 => "Thu".to_string(),
            4 => "Fri".to_string(),
            5 => "Sat".to_string(),
            6 => "Sun".to_string(),
            _ => "".to_string(),
        })
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLUE.mix(0.5).filled())
            .data(
                weekdays
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| (i as i32, day_count.get(&d).cloned().unwrap_or(0))),
            ),
    )?;

    root.present()?;
    println!("Saved histogram to {}", filename);

    Ok(())
}

fn create_histogram_hours(
    name: &str,
    hour_counts: &[u32; 24],
    max: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}-by-hour.png", name);
    let root = BitMapBackend::new(&filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = max.unwrap_or(*hour_counts.iter().max().unwrap_or(&0));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("distribuzione oraria di {}", name),
            ("sans-serif", 40),
        )
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(0..24, 0u32..max_count)?;

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

fn compute_winners(
    records: &[(NaiveDate, String)],
    period_label: fn(&NaiveDate) -> String,
) -> Vec<(String, String, u32)> {
    let mut periods: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for (date, name) in records {
        let key = period_label(date);
        *periods
            .entry(key)
            .or_default()
            .entry(name.clone())
            .or_insert(0) += 1;
    }
    let mut winners: Vec<_> = periods
        .into_iter()
        .map(|(period, persons)| {
            let (person, count) = persons.into_iter().max_by_key(|&(_, c)| c).unwrap();
            (period, person, count)
        })
        .collect();
    winners.sort_by(|a, b| a.0.cmp(&b.0));
    winners
}

fn render_winner_chart(
    filename: &str,
    title: &str,
    data: &[(String, String, u32)],
) -> Result<(), Box<dyn Error>> {
    if data.is_empty() {
        return Ok(());
    }

    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = data.iter().map(|(_, _, c)| *c).max().unwrap_or(1);
    let n = data.len();

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(5)
        .build_cartesian_2d(0usize..n, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_labels(n.min(20))
        .x_label_formatter(&|x| {
            if *x < data.len() {
                data[*x].0.clone()
            } else {
                String::new()
            }
        })
        .draw()?;

    let colors = [
        RED,
        BLUE,
        GREEN,
        MAGENTA,
        CYAN,
        RGBColor(255, 128, 0),
        RGBColor(128, 0, 128),
    ];
    let mut color_map: HashMap<&str, RGBColor> = HashMap::new();
    let mut ci = 0usize;

    for (i, (_, person, count)) in data.iter().enumerate() {
        let color = *color_map
            .entry(person.as_str())
            .or_insert_with(|| {
                let c = colors[ci % colors.len()];
                ci += 1;
                c
            });

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(color.mix(0.5).filled())
                .data(vec![(i, *count)]),
        )?;

        chart.draw_series(std::iter::once(Text::new(
            format!("{}", person),
            (i, *count + max_count / 20),
            ("sans-serif", 14).into_font().transform(FontTransform::Rotate90),
        )))?;
    }

    root.present()?;
    println!("Saved {}", filename);
    Ok(())
}

fn merge_vertical_images(
    image_paths: &Vec<&str>,
    output_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let padding = 10;
    let images: Vec<DynamicImage> = image_paths
        .iter()
        .map(|path| image::open(path).expect("failed to open immage"))
        .collect();

    let image_width = images[0].width();
    let image_height = images[0].height();
    let total_width = image_width;
    let total_height = (image_height + padding) * images.len() as u32 + padding;

    let mut combined_image = RgbaImage::new(total_width, total_height);

    let mut y_offset = 0;

    for img in images {
        combined_image.copy_from(&img, 0, y_offset)?;
        y_offset += image_height + padding;
    }

    combined_image.save(output_filename)?;
    println!("Saved combined histogram to {}", output_filename);
    Ok(())
}
