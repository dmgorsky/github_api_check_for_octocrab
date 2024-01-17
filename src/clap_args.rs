use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// YAML file with Github OpenAPI description
    #[arg(short, long, default_value = "api.github.com.2022-11-28.yaml")]
    pub input: String,

    /// Directory with [octocrab] source (*.rs) files (can have sub-dirs)
    #[arg(long)]
    pub sources: String,

    /// TSV file with information on URLs not mentioned in `sources`
    #[arg(short, long, default_value = "not-found.tsv")]
    pub output: String,

    /// TSV file with information (to check the project) on URLS that are mentioned in `sources`
    #[arg(short, long, default_value = "found-check.tsv")]
    pub found_report: String,
}
