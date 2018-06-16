use chrono::{Duration, Local, TimeZone, Utc};
use palette::rgb::Rgba;
use palette::Hsv;
use palette::RgbHue;
use pdf_canvas::graphicsstate::Color;
use pdf_canvas::{BuiltinFont, FontSource, Pdf};

use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::f32::consts;

use super::Report;
use interval::Interval;

use util::format_hms;

#[derive(Default)]
pub struct DefaultReport;

impl Report for DefaultReport {
    fn render(
        &self,
        config: &HashMap<String, String>,
        intervals: &Vec<Interval>,
        report_filename: &str,
    ) -> Result<Pdf, Box<Error>> {
        debug!(
            "Report span (Strings): {:?} - {:?}",
            config["temp.report.start"], config["temp.report.end"]
        );

        // Compute some statistics
        let format = "%Y%m%dT%H%M%SZ";
        let start_utc = Utc
            .datetime_from_str(&config["temp.report.start"], format)?
            .naive_utc();
        let end_utc = match config["temp.report.end"].as_str() {
            "" => Utc::now().naive_utc(),
            val => Utc.datetime_from_str(val, format)?.naive_utc(),
        };

        let start = Local.from_utc_datetime(&start_utc);
        let end = Local.from_utc_datetime(&end_utc);

        println!("Report start:\t{}", start.to_rfc2822());
        println!("Report end:\t{}", end.to_rfc2822());

        let mut total_time_logged = Duration::seconds(0);
        let mut unique_tag_sets: HashMap<BTreeSet<String>, Duration> = HashMap::new();

        for interval in intervals {
            total_time_logged = total_time_logged + interval.to_duration();

            if unique_tag_sets.contains_key(&interval.tags) {
                let previous_duration = unique_tag_sets[&interval.tags];
                unique_tag_sets.insert(
                    interval.tags.clone(),
                    previous_duration + interval.to_duration(),
                );
            } else {
                unique_tag_sets.insert(interval.tags.clone(), interval.to_duration());
            }
        }

        println!("Total time logged: {}", format_hms(&total_time_logged));

        let tag_set_count = unique_tag_sets.iter().count() as f32;
        println!("Unique tag set count: {}", tag_set_count);

        // How many radians apart each two hues need to be
        let colorspace_increment = 2.0 * consts::PI / tag_set_count;
        trace!("Colorpsace increment: {}", colorspace_increment);

        let mut current_color_radians = 0.0;

        // Shadow the map with a sorted one
        let mut unique_tag_sets: Vec<_> = unique_tag_sets.iter().collect();
        unique_tag_sets.sort_unstable_by(|item_a, item_b| item_a.1.cmp(item_b.1));

        let unique_tag_sets: Vec<_> = unique_tag_sets
            .iter()
            .map(|(tag_set, duration)| {
                let color_hsv = Hsv::new(RgbHue::from_radians(current_color_radians), 1.0, 0.75);
                let color_rgb: Rgba = color_hsv.into();
                let color_tuple: (f32, f32, f32, f32) = color_rgb.into_components();
                current_color_radians += colorspace_increment;
                (
                    tag_set,
                    duration,
                    (
                        (color_tuple.0 * 256.0) as u8,
                        (color_tuple.1 * 256.0) as u8,
                        (color_tuple.2 * 256.0) as u8,
                    ),
                )
            })
            .collect();

        let mut document = Pdf::create(report_filename)?;

        let page_dim = (180.0, 240.0);

        document.render_page(page_dim.0, page_dim.1, |canvas| {
            let font = BuiltinFont::Helvetica_Bold;
            let margin = 10.0;

            // Title
            let title = &format!("{} - {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
            let title_font_size = 10.0;
            let title_width = font.get_width(title_font_size, title) + 8.0;
            let title_y = page_dim.1 - 20.0;

            trace!("Computed title width: {}", title_width);

            canvas.set_stroke_color(Color::gray(0))?;
            canvas.set_line_width(0.5)?;
            canvas.line(
                (page_dim.0 - title_width) / 2.0,
                title_y - 6.0,
                (page_dim.0 + title_width) / 2.0,
                title_y - 6.0,
            )?;
            canvas.stroke()?;

            canvas.center_text(90.0, title_y, font, title_font_size, title)?;

            // Unique tag sets
            canvas.left_text(margin, title_y - 30.0, font, 8.0, "Time spent by tags")?;

            let unique_tag_initial_y = title_y - 40.0;
            let mut offset = 0.0;
            for (tag_set, duration, color) in &unique_tag_sets {
                canvas.set_fill_color(Color::rgb(color.0, color.1, color.2))?;
                canvas.rectangle(margin, unique_tag_initial_y - offset, 3.0, 3.0)?;
                canvas.fill()?;

                canvas.set_fill_color(Color::rgb(0, 0, 0))?;

                let pct =
                    duration.num_seconds() as f32 / total_time_logged.num_seconds() as f32 * 100.0;

                println!("{:?}: {} ({:.2}%)", tag_set, format_hms(&duration), pct);

                canvas.left_text(
                    margin * 2.0,
                    unique_tag_initial_y - offset,
                    font,
                    5.0,
                    &format!("{:?} ({:.2}%):", tag_set, pct),
                )?;
                canvas.right_text(
                    page_dim.0 - margin,
                    unique_tag_initial_y - offset,
                    font,
                    5.0,
                    &format_hms(duration),
                )?;
                offset += 6.0;
            }
            canvas.stroke()?;

            canvas.set_fill_color(Color::rgb(150, 0, 0))?;
            canvas.left_text(margin * 2.0, unique_tag_initial_y - offset, font, 5.0, "TOTAL:")?;
            canvas.right_text(
                page_dim.0 - margin,
                unique_tag_initial_y - offset,
                font,
                5.0,
                &format_hms(&total_time_logged),
            )?;
            canvas.fill()?;

            // Bar chart
            debug!("Starting the bar chart draw");
            let bar_chart_initial_coord = (margin, unique_tag_initial_y - (offset + 15.0));
            let bar_chart_dims = (page_dim.0 - 2.0 * margin, 10.0);

            let mut bar_start_x = bar_chart_initial_coord.0;
            for (_tag_set, duration, color) in unique_tag_sets.iter().rev() {
                let ratio = duration.num_seconds() as f32 / total_time_logged.num_seconds() as f32;
                let current_bar_width = bar_chart_dims.0 * ratio;

                canvas.set_fill_color(Color::rgb(color.0, color.1, color.2))?;
                canvas.rectangle(
                    bar_start_x,
                    bar_chart_initial_coord.1,
                    current_bar_width,
                    bar_chart_dims.1,
                )?;
                canvas.fill()?;

                bar_start_x += current_bar_width;
            }

            Ok(())
        })?;
        Ok(document)
    }
}
