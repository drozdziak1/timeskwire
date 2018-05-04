extern crate docopt;
extern crate env_logger;
extern crate libc;
extern crate serde_json;

#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::{BufReader, Read, Write};
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process;

use docopt::Docopt;
use serde_json::Value;

const USAGE: &'static str = "
TimeSkwire - a PDF render extension for TimeWarrior.

Usage:
  timeskwire [default] [output_filename]
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

fn main() {
    env_logger::init();

    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());

    match VERSION {
        Some(v) => info!("TimeSkwire {}", v),
        None => warn!("Could not retrieve TimeSkwire version"),
    };

    if args.get_bool("--version") {
        eprintln!("{}", VERSION.unwrap_or("unknown"));
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

        dir.push("timeskwire");

        init(dir.as_path(), args.get_bool("--force")).unwrap_or_else(|e| {
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

    let (config, times) = parse_input(BufReader::new(io::stdin())).unwrap();

    println!(
        "TimeWarrior version: {}",
        config
            .get("temp.version")
            .unwrap_or(&String::from("unknown"))
    );

    for item in times {
        println!("Item: {}", item);
    }
}

fn init(extension_dir: &Path, force: bool) -> Result<(), Box<Error>> {
    if !extension_dir.is_dir() {
        writeln!(
            io::stderr(),
            "timeskwire: {}: No such file or directory",
            extension_dir.to_str().unwrap()
        ).unwrap();
        process::exit(libc::EXIT_FAILURE);
    };

    let src = env::current_exe()?;
    let dst = fs::canonicalize(extension_dir)?;

    if dst.exists() && force {
        debug!("`force` is true, removing target file");
        fs::remove_file(dst.as_path())?;
    }

    info!("Bootstrapping {:?} at {:?}", src, dst);
    unix::fs::symlink(src.as_path(), dst.as_path())?;
    Ok(())
}
fn parse_input<'a, T: Read>(
    mut input: BufReader<T>,
) -> Result<(HashMap<String, String>, Vec<Value>), Box<Error>> {
    let sections: Vec<String> = {
        let mut input_buf = String::new();

        input.read_to_string(&mut input_buf).unwrap();
        input_buf
            .split("\n\n")
            .map(|section| String::from(section))
            .collect()
    };

    let mut config = HashMap::new();
    // Parse config value section
    for line in sections[0].lines() {
        let entry: Vec<&str> = line.splitn(2, ": ").collect();
        debug!("Got key '{}' with value '{}'.", entry[0], entry[1]);

        config.insert(String::from(entry[0]), String::from(entry[1]));
    }

    Ok((config, serde_json::from_str(&sections[1])?))
}
