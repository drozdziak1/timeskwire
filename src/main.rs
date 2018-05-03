extern crate docopt;
extern crate env_logger;
extern crate libc;

#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process;

use docopt::Docopt;

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
            let mut custom = PathBuf::from(args.get_str("<extension_dir>"));

            if custom.is_dir() {
                if custom.is_relative() {
                    debug!("Custom dir not absolute, appending to current directory...");
                    let mut current = env::current_dir().unwrap();
                    current.push(custom);
                    custom = current;
                }
                custom
            } else {
                writeln!(
                    io::stderr(),
                    "timeskwire: {}: No such file or directory",
                    custom.to_str().unwrap()
                ).unwrap();
                process::exit(libc::EXIT_FAILURE);
            }
        } else {
            let mut default = env::home_dir().unwrap();
            default.push(DEFAULT_EXTENSION_SUBDIR);

            default
        };

        dir.push("timeskwire");

        bootstrap(dir.as_path(), args.get_bool("--force")).unwrap_or_else(|e| {
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
    let sections: Vec<String> = {
        let mut input_buf = String::new();

        io::stdin().read_to_string(&mut input_buf).unwrap();
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

        config.insert(entry[0], entry[1]);
    }

    println!("TimeWarrior version: {}", config.get("temp.version").unwrap_or(&"unknown"));
}

fn bootstrap(dst: &Path, force: bool) -> Result<(), Box<Error>> {
    let src_buf = env::current_exe()?;

    if dst.exists() && force {
        debug!("`force` is true, removing target file");
        fs::remove_file(dst)?;
    }

    info!("Bootstrapping {:?} at {:?}", src_buf, dst);
    unix::fs::symlink(src_buf.as_path(), &dst)?;
    Ok(())
}
