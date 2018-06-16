pub mod default;

use pdf_canvas::Pdf;

use std::collections::HashMap;
use std::error::Error;

use interval::Interval;

// Reexports
pub use self::default::DefaultReport;

pub trait Report {
    fn render(
        &self,
        config: &HashMap<String, String>,
        intervals: &Vec<Interval>,
        report_filename: &str,
    ) -> Result<Pdf, Box<Error>>;
}
