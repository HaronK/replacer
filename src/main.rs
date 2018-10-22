use colored::*;
use failure::{Error, ResultExt};
use rayon::prelude::*;
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
    /// Pattern string (rust regex).
    #[structopt(parse(from_str))]
    pattern: String,
    /// Show replaced or matched lines.
    #[structopt(short = "d", long = "show-diff")]
    show_diff: bool,
    /// Replacement string (rust regex). Do only pattern matching if not specified.
    #[structopt(short = "r", long = "replace")]
    replacement: Option<String>,
    /// Input file or starting directory. Searches in the current directory if not specified.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
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

fn patch_file(
    path: &Path,
    re: &Regex,
    replacement: &Option<String>,
    show_diff: bool,
) -> Result<usize, Error> {
    let old_bytes = fs::read(path).context(format!("Cannot read from the file"))?;
    let old_text = String::from_utf8_lossy(&old_bytes);

    if let Some(replace_str) = replacement {
        let new_text = re.replace_all(&old_text, replace_str.as_str());

        if let Owned(s) = new_text {
            println!("Changes in file: {:?}", path);

            fs::write(path, s.as_bytes())?;

            if show_diff {
                print_diff(&old_text, &s);
            }

            return Ok(1);
        }
    } else {
        println!("Matches in file: {:?}", path);

        let mut matches_count = 0;
        for pos in re.find_iter(&old_text) {
            if show_diff {
                println!("{}", pos.as_str().red());
            }

            matches_count += 1;
        }

        if matches_count > 0 {
            println!("Matches count: {}", matches_count);
            return Ok(1);
        }
    }
    Ok(0)
}

fn patch_dir<P: AsRef<Path>>(
    path: P,
    re: &Regex,
    replacement: &Option<String>,
    show_diff: bool,
) -> Result<usize, Error> {
    // NOTE: walkdir doesn't support rayon so we just collect file paths and then use rayon to process them in parallel
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                files.push(path_str.to_string());
            }
        }
    }

    let files_processed = files
        .par_iter()
        .map(|file| {
            patch_file(Path::new(file), re, replacement, show_diff)
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

    let re = Regex::new(&opt.pattern)?;
    let files_processed = if let Some(path_buf) = opt.input {
        let path = path_buf.as_path().clone();
        if path.exists() {
            if path.is_file() {
                patch_file(path, &re, &opt.replacement, opt.show_diff)
                    .context(format!("File: {:?}", path))?
            } else if path.is_dir() {
                patch_dir(path, &re, &opt.replacement, opt.show_diff)?
            } else {
                eprintln!("Unknown type of the file {:?}", path);
                0
            }
        } else {
            eprintln!("Path {:?} doesn't exist!", path);
            0
        }
    } else {
        patch_dir("./", &re, &opt.replacement, opt.show_diff)?
    };

    if opt.replacement.is_some() {
        println!("Total replaced files: {}", files_processed);
    } else {
        println!("Total matched files: {}", files_processed);
    }

    Ok(())
}
