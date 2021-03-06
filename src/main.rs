#[macro_use]
extern crate failure;

use colored::*;
use failure::{Error, ResultExt};
use rayon_cond::CondIterator;
use regex::Regex;
use std::borrow::Cow::*;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Replace text in the files using regex pattern.\nSearch in the specified file or in all files of the folder recursively.\nSupports multiline pattern replacement."
)]
struct Opt {
    /// Show replaced or matched lines.
    #[structopt(short = "d", long = "show-diff")]
    show_diff: bool,
    /// Pattern string for the file name (rust regex).
    #[structopt(short = "f", long = "file")]
    file_pattern: Option<String>,
    /// Pattern string for the text (rust regex).
    #[structopt(parse(from_str))]
    text_pattern: String,
    /// Replacement string (rust regex). Do only pattern matching if not specified.
    #[structopt(short = "r", long = "replace")]
    replacement: Option<String>,
    /// Input files and/or starting directories. Searches in the current directory if not specified.
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,
}

fn print_diff(left: &str, right: &str) {
    for diff in diff::lines(left, right) {
        match diff {
            diff::Result::Left(l) => println!("-{}", l.red()),
            diff::Result::Right(r) => println!("+{}", r.green()),
            _ => {}
        }
    }
}

fn show_matches_with_diff(path: &Path, re_text: &Regex, text: &str) -> usize {
    println!("Matches in file: {:?}", path);

    let mut matches_count = 0;
    for pos in re_text.find_iter(&text) {
        println!("{}", pos.as_str().red());

        matches_count += 1;
    }

    if matches_count > 0 {
        println!("Matches count: {}", matches_count);
    }

    matches_count
}

fn show_matches(path: &Path, re_text: &Regex, text: &str) -> usize {
    let matches_count = re_text.find_iter(&text).count();
    println!("{} matches in file: {:?}", matches_count, path);
    matches_count
}

fn process_file(
    path: &Path,
    re_file: &Option<Regex>,
    re_text: &Regex,
    replacement: &Option<String>,
    show_diff: bool,
) -> Result<usize, Error> {
    if let Some(re) = re_file {
        if let Some(path_str) = path.to_str() {
            if !re.is_match(path_str) {
                return Ok(0);
            }
        } else {
            bail!("Path is not valid: {:?}", path);
        }
    }

    let old_bytes = fs::read(path).context("Cannot read from the file")?;
    let old_text = String::from_utf8_lossy(&old_bytes);

    if let Some(replace_str) = replacement {
        let new_text = re_text.replace_all(&old_text, replace_str.as_str());

        if let Owned(s) = new_text {
            println!("Changes in file: {:?}", path);

            fs::write(path, s.as_bytes())?;

            if show_diff {
                print_diff(&old_text, &s);
            }

            return Ok(1);
        }
    } else {
        let matches_count = if show_diff {
            show_matches_with_diff(path, re_text, &old_text.into_owned())
        } else {
            show_matches(path, re_text, &old_text.into_owned())
        };

        return Ok(if matches_count > 0 { 1 } else { 0 });
    }
    Ok(0)
}

fn collect_files(dir: &str, files: &mut Vec<String>) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                files.push(path_str.to_string());
            }
        }
    }
}

fn process_files(
    files: &[String],
    re_file: &Option<Regex>,
    re_text: &Regex,
    replacement: &Option<String>,
    show_diff: bool,
) -> Result<usize, Error> {
    let files_processed = CondIterator::new(files, !show_diff)
        .map(|file| {
            process_file(Path::new(file), re_file, re_text, replacement, show_diff)
                .context(format!("File: {:?}", file))
        })
        .inspect(|result| {
            if let Err(err) = result {
                println!("{}", err);
            }
        })
        .filter_map(Result::ok)
        .sum();

    Ok(files_processed)
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let mut files = vec![];
    if !opt.inputs.is_empty() {
        for path_buf in opt.inputs {
            let path = path_buf.as_path();
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    if path.is_file() {
                        files.push(path_str.to_string());
                    } else if path.is_dir() {
                        collect_files(path_str, &mut files);
                    } else {
                        println!("Unknown type of the file {:?}", path);
                    }
                }
            } else {
                println!("Path {:?} doesn't exist!", path);
            }
        }
    } else {
        collect_files("./", &mut files);
    };

    let re_file = if let Some(p) = opt.file_pattern {
        Some(Regex::new(&p).context("Can't parse text pattern.")?)
    } else {
        None
    };
    let re_text = Regex::new(&opt.text_pattern).context("Can't parse text pattern.")?;

    let files_processed =
        process_files(&files, &re_file, &re_text, &opt.replacement, opt.show_diff)?;

    if opt.replacement.is_some() {
        println!("Total replaced files: {}", files_processed);
    } else {
        println!("Total matched files: {}", files_processed);
    }

    Ok(())
}
