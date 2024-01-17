# github api simple check for octocrab

The project:
* uses github api yaml from https://github.com/github/rest-api-description/tree/main/descriptions/api.github.com (downloaded locally)
* builds a list of URLs provided by the API
* looks for the URLs from the list within `octocrab` sources (downloaded locally) 
* reports with URLs not found (via `regex`es) in the sources - generates `not-found.tsv` with not found URLs
* generates also `found.tsv` with path/url parameters for each api found in the `octocrab` sources to check
* `clap` here provides the following params description:
```text
Usage: github_yaml [OPTIONS] --sources <SOURCES>

Options:
  -i, --input <INPUT>                YAML file with Github OpenAPI description [default: api.github.com.2022-11-28.yaml]
      --sources <SOURCES>            Directory with [octocrab] source (*.rs) files (can have sub-dirs)
  -o, --output <OUTPUT>              TSV file with information on URLs not mentioned in `sources` [default: not-found.tsv]
  -f, --found-report <FOUND_REPORT>  TSV file with information (to check the project) on URLS that are mentioned in `sources` [default: found-check.tsv]
  -h, --help                         Print help
```

So, being run as 

`cargo run -- --sources ~/rust/octocrab/src` ,

generates in `not-found.tsv`:

| Tag     | URL                                               | Parameters                                                                                                                                                                                                                         |
|---------|---------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| actions | /orgs/{org}/actions/runners/registration-token    | post -> [reqd]org: string (in: path)                                                                                                                                                                                               |
| actions | /repos/{owner}/{repo}/actions/runners/{runner_id} | get -> [reqd]owner: string (in: path), [reqd]repo: string (in: path), [reqd]runner_id: integer (in: path); delete -> [reqd]owner: string (in: path), [reqd]repo: string (in: path), [reqd]runner_id: integer (in: path)            |
| actions | /orgs/{org}/actions/secrets/{secret_name}         | get -> [reqd]org: string (in: path), [reqd]secret_name: string (in: path); put -> [reqd]org: string (in: path), [reqd]secret_name: string (in: path); delete -> [reqd]org: string (in: path), [reqd]secret_name: string (in: path) |

and so on.



crates used:
```text
anyhow
serde_yaml
regex
walkdir
rayon
indicatif
clap
itertools

```
