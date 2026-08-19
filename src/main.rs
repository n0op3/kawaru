use std::{
    fs::{read, write},
    path::PathBuf,
};

use clap::Parser;
use file_identify::tags_from_path;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::bytes::Regex;
use walkdir::DirEntry;

#[derive(Parser)]
#[command(
    version,
    about = "Simple find-and-replace supporting both text and binary files"
)]
struct Args {
    /// Whether to also replace the text in binary files
    #[arg(short, long)]
    binary: bool,

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

    let exclusion_regex = args.exclude.map(|regex| Regex::new(&regex).unwrap());
    let regex = Regex::new(&args.regex).unwrap();

    let replacement = args.replacement.into_bytes();
    walkdir::WalkDir::new(
        args.directory
            .map(PathBuf::from)
            .unwrap_or(std::env::current_dir().expect("couldn't get current_dir")),
    )
    .into_iter()
    .filter_map(|entry| entry.ok())
    .filter(|file| {
        file.metadata().unwrap().is_file()
            && (args.binary || tags_from_path(file.path()).unwrap().contains("text"))
            && exclusion_regex
                .as_ref()
                .is_none_or(|regex| !regex.is_match(file.file_name().as_encoded_bytes()))
    })
    .collect::<Vec<DirEntry>>()
    .par_iter()
    .for_each(|file| {
        println!("Replacing text in {:?}", file.file_name());

        let contents = read(file.path()).unwrap();
        write(file.path(), regex.replace_all(&contents, &replacement)).unwrap();
    });
}
