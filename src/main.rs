extern crate chrono;
extern crate docopt;
extern crate env_logger;
extern crate libc;
extern crate palette;
extern crate pdf_canvas;
extern crate serde_json;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod interval;
mod reports;
mod util;

use chrono::{Local, TimeZone, Utc};
use docopt::Docopt;
use serde_json::Value;

use std::collections::{BTreeSet, HashMap};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::{BufReader, Read, Write};
use std::os::unix;
use std::path::PathBuf;
use std::process;

use interval::Interval;
use reports::{DefaultReport, Report};

const USAGE: &'static str = "
TimeSkwire - a PDF render extension for TimeWarrior.

Without arguments, TimeSkwire will be waiting for input from TimeWarrior's extension API. You can
read more about it at https://taskwarrior.org/docs/timewarrior/api.html

Usage:
  timeskwire
  timeskwire init [<extension_dir> (-f | --force)]
  timeskwire (-h | --help)
  timeskwire --version

Options:
  <extension_dir>   Where to initialize TimeSkwire (~/.timewarrior/extensions/ by default).
  -f --force        Forces initialization if the symlink already exists
  -h --help         Shows this screen.
  --version         Prints the version of your TimeSkwire.
";

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const DEFAULT_EXTENSION_SUBDIR: &'static str = ".timewarrior/extensions/";
const DEFAULT_REPORT_FILENAME: &'static str = "report.pdf";

fn main() {
    env_logger::init();

    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());

    if args.get_bool("--version") {
        println!("{}", VERSION.unwrap_or("unknown"));
        process::exit(libc::EXIT_SUCCESS);
    }

    if args.get_bool("init") {
        let mut dir = if args.get_bool("<extension_dir>") {
            PathBuf::from(args.get_str("<extension_dir>"))
        } else {
            let mut default = env::home_dir().unwrap();
            default.push(DEFAULT_EXTENSION_SUBDIR);

            default
        };
        trace!("Using extension dir: {:?}", dir);

        init(&mut dir, args.get_bool("--force")).unwrap_or_else(|e| {
            writeln!(
                io::stderr(),
                "timeskwire: init: Could not symlink to {:?}: {}",
                dir,
                e.to_string()
            ).unwrap();
            process::exit(libc::EXIT_FAILURE);
        });

        println!("Init OK. Check that your TimeWarrior sees timeskwire with `timew extensions`.");
        process::exit(libc::EXIT_SUCCESS);
    }

    let (config, intervals) = parse_input(BufReader::new(io::stdin())).unwrap();

    println!(
        "TimeWarrior version {}",
        config
            .get("temp.version")
            .unwrap_or(&String::from("unknown"))
    );
    println!("TimeSkwire version {}", VERSION.unwrap_or("unknown"));

    let report_kind: String =
        env::var("TIMESKWIRE_REPORT").unwrap_or(match config.get("timeskwire.report.kind") {
            Some(s) => s.to_owned(),
            None => {
                eprintln!("Warning: No report choice made, using \"default\"");
                String::from("default")
            }
        });

    let report: Box<Report> = match report_kind.as_str() {
        "default" => Box::new(DefaultReport {}),
        _ => Box::new(DefaultReport {}),
    };

    let doc = report
        .render(
            &config,
            &intervals,
            match config.get("timeskwire.report.filename") {
                Some(name) => &name,
                None => {
                    info!(
                        "No report filename defined, falling back to {}",
                        DEFAULT_REPORT_FILENAME
                    );
                    DEFAULT_REPORT_FILENAME
                }
            },
        )
        .unwrap();

    doc.finish().unwrap();
}

fn init(extension_path: &mut PathBuf, force: bool) -> Result<(), Box<Error>> {
    if !extension_path.is_dir() {
        writeln!(
            io::stderr(),
            "timeskwire: {}: No such file or directory",
            extension_path.to_str().unwrap()
        ).unwrap();
        process::exit(libc::EXIT_FAILURE);
    };

    extension_path.push("timeskwire");

    let src = env::current_exe()?;

    if extension_path.exists() && force {
        debug!("`force` is true, removing target file");
        fs::remove_file(extension_path.as_path())?;
    }

    info!("Bootstrapping {:?} at {:?}", src, extension_path);
    unix::fs::symlink(src.as_path(), extension_path.as_path())?;
    Ok(())
}

fn parse_input<'a, T: Read>(
    mut input: BufReader<T>,
) -> Result<(HashMap<String, String>, Vec<Interval>), Box<Error>> {
    let sections: Vec<String> = {
        let mut input_buf = String::new();

        input.read_to_string(&mut input_buf)?;
        input_buf
            .split("\n\n")
            .map(|section| String::from(section))
            .collect()
    };

    let mut config = HashMap::new();
    // Parse config value section
    for line in sections[0].lines() {
        let entry: Vec<&str> = line.splitn(2, ": ").collect();
        trace!("Got key '{}' with value '{}'.", entry[0], entry[1]);

        config.insert(String::from(entry[0]), String::from(entry[1]));
    }

    let values: Vec<Value> = serde_json::from_str(&sections[1])?;

    let mut intervals = Vec::new();

    for value in values {
        let mut tags: BTreeSet<String> = BTreeSet::new();

        let tags_raw = value["tags"].as_array().unwrap();
        for tag in tags_raw {
            tags.insert(String::from(tag.as_str().unwrap()));
        }

        let format = "%Y%m%dT%H%M%SZ";
        let start_str = value["start"].as_str().unwrap();
        trace!("Parsing start date {:?}", start_str);
        let start_utc = Utc.datetime_from_str(start_str, format)?.naive_utc();

        // There's no "end" key if there's unfinished logging in progress; use now in that case
        let end_utc = match value.get("end") {
            Some(val) => Utc
                .datetime_from_str(val.as_str().unwrap(), format)?
                .naive_utc(),
            None => {
                let end = Utc::now().naive_utc();
                println!(
                    "Time logging still in progress, using now ({:?}) as end",
                    end
                );
                end
            }
        };

        intervals.push(Interval {
            start: Local.from_utc_datetime(&start_utc),
            end: Local.from_utc_datetime(&end_utc),
            tags: tags,
        });
    }

    Ok((config, intervals))
}
