use failure::{Error, ResultExt};
use regex::Regex;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Replace text in the files using regex pattern.\nSearch in the specified file or in all files of the folder recursively.\nSupports multiline pattern replacement."
)]
struct Opt {
    /// Pattern string (rust regex)
    #[structopt(parse(from_str))]
    pattern: String,
    /// Replacement string (rust regex)
    #[structopt(parse(from_str))]
    replacement: String,
    /// Input file or starting directory. Searches in the current directory if not specified.
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn patch_file(path: &Path, re: &Regex, replacement: &String) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .context(format!("Cannot open file"))?;
    let mut old_text = String::new();
    file.read_to_string(&mut old_text)
        .context(format!("Cannot read from the file"))?;

    //let old_text = String::from_utf8_lossy(&fs::read(path).context(format!("Cannot read from the file"))?).parse()?;

    if re.is_match(&old_text) {
        let new_text = re.replace_all(&old_text, replacement.as_str());

        fs::write(path, new_text.as_bytes())?;
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let re = Regex::new(&opt.pattern)?;
    if let Some(path_buf) = opt.input {
        let path = path_buf.as_path().clone();
        if path.exists() {
            if path.is_file() {
                patch_file(path, &re, &opt.replacement).context(format!("File: {:?}", path))?;
            } else if path.is_dir() {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        patch_file(entry.path(), &re, &opt.replacement)
                            .context(format!("File: {:?}", entry.path()))?;
                    }
                }
            } else {
                eprintln!("Unknown type of the file {:?}", path);
            }
        } else {
            eprintln!("Path {:?} doesn't exist!", path);
        }
    } else {
        for entry in WalkDir::new("./").into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                patch_file(entry.path(), &re, &opt.replacement)
                    .context(format!("File: {:?}", entry.path()))?;
            }
        }
    }
    Ok(())
}
