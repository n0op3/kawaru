use std::{
    fs::read,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use file_identify::tags_from_path;
use ignore::{DirEntry, WalkBuilder};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::bytes::Regex;
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(
    version,
    about = "Simple find-and-replace supporting both text and binary files",
    disable_help_flag = true
)]
struct Args {
    /// Whether to also replace the text in binary files
    #[arg(short, long)]
    binary: bool,

    /// Whether to alter hidden files
    #[arg(short, long)]
    hidden: bool,

    /// Respects .gitignore and .git/info/exclude
    #[arg(short, long)]
    git: bool,

    /// Regex to exclude files
    #[arg(short, long)]
    exclude: Option<String>,

    /// Regex to match
    regex: String,

    /// Text to replace with
    replacement: String,

    /// Directory or file to run on, if none, runs in the current one
    directory: Option<String>,
}

fn main() {
    let args = Args::parse();

    let exclusion_regex = args
        .exclude
        .map(|regex| Regex::new(&regex).expect("failed to compile the regex"));
    let regex = Regex::new(&args.regex).expect("failed to compile the regex");

    let replacement = args.replacement.into_bytes();

    WalkBuilder::new(
        args.directory
            .map(PathBuf::from)
            .unwrap_or(std::env::current_dir().expect("couldn't get current_dir")),
    )
    .hidden(args.hidden)
    .git_ignore(args.git)
    .git_exclude(args.git)
    .parents(true)
    .ignore(true)
    .build()
    .filter_map(|entry| entry.ok())
    .filter(|file| {
        (args.hidden || !file.file_name().to_string_lossy().starts_with("."))
            && file.metadata().is_ok_and(|metadata| metadata.is_file())
            && (args.binary || tags_from_path(file.path()).is_ok_and(|tags| tags.contains("text")))
            && exclusion_regex
                .as_ref()
                .is_none_or(|regex| !regex.is_match(file.file_name().as_encoded_bytes()))
    })
    .collect::<Vec<DirEntry>>()
    .par_iter()
    .map(|file| -> Result<_, (String, anyhow::Error)> {
        println!("Replacing text in {:?}", file.file_name());

        let path = file.path().to_string_lossy().to_string();

        let result = (|| -> anyhow::Result<()> {
            let contents = read(file.path())?;

            let mut tmp = NamedTempFile::new_in(file.path().parent().unwrap_or(Path::new(".")))?;

            tmp.write_all(&regex.replace_all(&contents, &replacement))?;
            tmp.persist(file.path())?;

            Ok(())
        })();

        result.map_err(|e| (path, e))?;

        Ok(())
    })
    .filter_map(|result| result.err())
    .for_each(|error| eprintln!("Could not process {}: {:?}", error.0, error.1));
}
