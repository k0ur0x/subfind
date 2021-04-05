use ansi_term::Color::{Blue, Green, Red};
use clap::{App, Arg};
use regex::{Match, Regex};
use std::{env, error::Error, ffi::OsStr, fs, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

mod options {
    pub const DIR: &str = "directory";
    pub const PATTERN: &str = "pattern";
    pub const RECURSIVE: &str = "recursive";
}

fn main() -> Result<()> {
    let current_dir = env::current_dir()?;
    let current_dir = match current_dir.to_str() {
        Some(path) => path,
        None => panic!("could not get the current working directory."),
    };

    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg(
            Arg::with_name(options::PATTERN)
                .value_name("PATTERN")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(options::DIR)
                .short("d")
                .long("directory")
                .value_name("DIR")
                .help("set a directory to search for the subtitles")
                .default_value(current_dir)
                .hide_default_value(true),
        )
        .arg(
            Arg::with_name(options::RECURSIVE)
                .short("r")
                .long("recursive")
                .help("search recursively"),
        )
        .get_matches();

    let pattern = matches.value_of(options::PATTERN).unwrap();
    let path = matches.value_of(options::DIR);
    let recursive = matches.is_present(options::RECURSIVE);

    let regex = Regex::new(pattern)?;
    subs(&regex, path, recursive)
}

fn subs(regex: &Regex, path: Option<&str>, recursive: bool) -> Result<()> {
    if let Some(path) = path {
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;

            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() && recursive {
                    subs(regex, entry.path().to_str(), true)?
                } else if file_type.is_file() {
                    if let Err(error) = find(regex, entry.path()) {
                        eprintln!(
                            "{}: {}",
                            Red.paint(error.to_string()),
                            entry.path().display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn find(regex: &Regex, path: PathBuf) -> Result<()> {
    let items = srtparse::from_file(&path)?;

    print_file_name(path.file_stem());

    for item in items {
        let text = item.text;

        if let Some(mat) = regex.find(&text) {
            print_match(&text, &mat);
        }
    }

    Ok(())
}

fn print_file_name(file_name: Option<&OsStr>) {
    let file_name = file_name
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .unwrap_or("");

    println!("{}", Blue.paint(file_name));
}

fn print_match(text: &str, mat: &Match) {
    println!(
        "{}{}{}",
        &text[..mat.start()],
        Green.paint(&text[mat.start()..mat.end()]),
        &text[mat.end()..]
    );
}
