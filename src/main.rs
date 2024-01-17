use std::sync::{Arc, Mutex};

use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use regex::Regex;

use crate::out_utils::OutTsvWriter;
use crate::sources_reader::SourcesReader;
use crate::swagger_parser::SwaggerYamlParser;

mod clap_args;
mod out_utils;
mod sources_reader;
mod swagger_parser;

fn main() -> anyhow::Result<()> {
    let arg = clap_args::Args::parse();
    let github_swagger = SwaggerYamlParser::read_file(arg.input.as_str());
    let urls = github_swagger.get_urls_regexes();
    let urls_count = urls.len();

    let sources = SourcesReader::open_folder(arg.sources.as_str());
    let file_names = sources.get_files_list();
    println!("Sorting out missing URLS...");
    let not_found: Arc<Mutex<Vec<Regex>>> = Arc::new(Mutex::new(vec![]));
    let found: Arc<Mutex<Vec<Regex>>> = Arc::new(Mutex::new(vec![]));

    urls.into_par_iter()
        .progress_count(urls_count as u64)
        .for_each(|search_regex| {
            let found_mention = |file| {
                let file_contents = sources.read_file(file);
                let found_in_file = file_contents
                    .into_iter()
                    .par_bridge()
                    .any(|line| search_regex.find(line.as_str()).is_some());
                found_in_file
            };
            if file_names.par_iter().any(found_mention) {
                not_found.lock().unwrap().push(search_regex)
            } else {
                found.lock().unwrap().push(search_regex)
            }
        });

    // let filtered = urls.into_par_iter()
    //     .progress_count(urls_count as u64)
    //     .filter(|search_regex| {
    //         file_names.par_iter().any(|file| {
    //             let file_contents = sources.read_file(file);
    //             let found_in_file = file_contents.into_iter().par_bridge().any(|line| search_regex.find(line.as_str()).is_some());
    //             found_in_file
    //         })
    //     }).collect::<Vec<Regex>>();

    println!("Preparing report on not_found...");
    let report_not_found = github_swagger.report_on_urls(&not_found.lock().unwrap());
    let out_not_found = OutTsvWriter::new(arg.output);
    let _ = out_not_found.write_to_csv(&report_not_found);
    println!("not found: {}", not_found.lock().unwrap().len());

    println!("Preparing report on found...");
    let report_found = github_swagger.report_on_urls(&found.lock().unwrap());
    let out_found = OutTsvWriter::new(arg.found_report);
    let _ = out_found.write_to_csv(&report_found);
    println!("found: {}", found.lock().unwrap().len());

    Ok(())
}

#[cfg(test)]
mod tests {
    // use rayon::prelude::IntoParallelRefIterator;
    // use regex::Regex;

    // use crate::{read_folder_contents_into_strings, transform_into_regex};

    // #[test]
    // fn test_regex_replace() {
    //     let mut tests = vec![];
    //     tests.push(("/zen", "/zen"));
    //     tests.push(("/orgs/{org}/personal-access-tokens/{pat_id}/repositories", "/orgs/(\\{\\w*})/personal-access-tokens/(\\{\\w*})/repositories"));
    //     tests.push(("/teams/{team_id}/teams", "/teams/(\\{\\w*})/teams"));
    //     tests.push(("/users/{username}/packages/{package_type}/{package_name}", "/users/(\\{\\w*})/packages/(\\{\\w*})/(\\{\\w*})"));
    //     for (src, reslt) in tests {
    //         assert_eq!(transform_into_regex(src), reslt);
    //     }
    // }

    // #[test]
    // fn test_regex_search() {
    //     let files = read_folder_contents_into_strings("/Users/dhorskyi/tmp/fil2"/*"/Users/dhorskyi/rust/octocrab/src"*/);
    //     let mut file_strings = files.par_iter().flat_map(|file_string| file_string.lines());
    //     // for fs in file_strings {println!("{fs:?}")};
    //     // dbg!(file_strings);
    //     let search_regex = Regex::new(r"/orgs/(\{\w*})/actions/secrets/(\{\w*})").unwrap();
    //     let found = file_strings.any(|line| search_regex.find(line).is_some());
    //     assert_eq!(found, true);
    // }
}
