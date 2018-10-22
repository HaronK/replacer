use failure::{Error, ResultExt};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;
use rayon::prelude::*;

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
    let old_bytes = fs::read(path).context(format!("Cannot read from the file"))?;
    let old_text = String::from_utf8_lossy(&old_bytes);

    if re.is_match(&old_text) {
        let new_text = re.replace_all(&old_text, replacement.as_str());

        fs::write(path, new_text.as_bytes())?;

        println!("File changed: {:?}", path);
    }
    Ok(())
}

fn patch_dir<P: AsRef<Path>>(path: P, re: &Regex, replacement: &String) -> Result<(), Error> {
    // NOTE: walkdir doesn't support rayon so we just collect file paths and then use rayon to process them in parallel
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                files.push(path_str.to_string());
            }
            // patch_file(entry.path(), re, replacement)
            //     .context(format!("File: {:?}", entry.path()))?;
        }
    }
    //TODO: collect errors
    files.par_iter().map(|file| patch_file(Path::new(file), re, replacement).context(format!("File: {:?}", file)));
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
                patch_dir(path, &re, &opt.replacement)?;
            } else {
                eprintln!("Unknown type of the file {:?}", path);
            }
        } else {
            eprintln!("Path {:?} doesn't exist!", path);
        }
    } else {
        patch_dir("./", &re, &opt.replacement)?;
    }
    Ok(())
}