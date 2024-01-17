use std::fs::File;
use std::io::{BufRead, BufReader};

use rayon::prelude::*;
use walkdir::WalkDir;

pub struct SourcesReader {
    src_dir: String,
}

impl SourcesReader {
    pub fn open_folder(folder_path: &str) -> Self {
        Self {
            src_dir: String::from(folder_path),
        }
    }

    pub fn get_files_list(&self /*, folder_path: &str*/) -> Vec<String> {
        WalkDir::new(self.src_dir.as_str())
            .follow_links(true)
            .into_iter()
            .par_bridge()
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().is_dir())
            .filter(|e| {
                e.path()
                    .file_name()
                    .is_some_and(|nm| nm.to_str().is_some_and(|s| !s.starts_with('.')))
            })
            .map(|entry| String::from(entry.path().to_str().unwrap()))
            .collect()
    }

    pub fn read_file(&self, file: &String) -> impl Iterator<Item = String> {
        BufReader::new(File::open(file).unwrap())
            .lines()
            .map(Result::unwrap)
    }
}
