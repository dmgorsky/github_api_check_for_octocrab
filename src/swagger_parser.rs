use std::fmt::{Display, Formatter};
use std::fs::File;

use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use serde_yaml::Value;

const DELIM: char = '\t';

pub struct ReportRecord(pub(crate) String, String, String);

impl ReportRecord {
    pub fn tsv_header() -> String {
        format!("Tag{DELIM}URL{DELIM}Parameters\n")
    }
}

impl From<ReportRecord> for String {
    fn from(value: ReportRecord) -> Self {
        format!("{}{DELIM}{}{DELIM}{}", value.0, value.1, value.2)
    }
}

impl Display for ReportRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}{DELIM}{}{DELIM}{}", self.0, self.1, self.2)
    }
}

pub struct SwaggerYamlParser {
    paths_value: Box<Value>,
    all_yaml_data: Box<Value>,
}

impl<'a> SwaggerYamlParser {
    pub fn read_file(yaml_path: &str) -> Self {
        let f = File::open(yaml_path).unwrap();
        let data: Value = serde_yaml::from_reader(f).unwrap();
        let swagger_paths = Box::new(data.get("paths").unwrap().clone());
        Self {
            paths_value: swagger_paths,
            all_yaml_data: Box::new(data),
        }
    }

    pub fn get_urls(&self /*, paths: &Value*/) -> Vec<String> {
        const NO_URL: &str = "";
        match &*self.paths_value {
            Value::Mapping(m) => {
                m.keys()
                    .map(|key| String::from(key.as_str().unwrap_or(NO_URL)))
            }
            .collect(),
            _ => Vec::new(),
        }
    }

    pub fn get_urls_regexes(&self) -> Vec<Regex> {
        self.get_urls()
            .par_iter()
            .map(|url| self.transform_into_regex(url.as_str()))
            .collect::<Vec<Regex>>()
    }

    pub(crate) fn report_on_urls(&self, not_found: &Vec<Regex>) -> Vec<ReportRecord> {
        //

        const NO_TAG: &str = "";
        const NO_URL: &str = "";
        match self.paths_value.as_ref() {
            Value::Mapping(m) => {
                m.keys()
                    .par_bridge()
                    // analyze only yaml path keys that are in 'not found' collection
                    .filter(|key_url| {
                        key_url.as_str().is_some_and(|ku| {
                            let compare_regex = self.transform_into_regex(ku);
                            not_found
                                .par_iter()
                                .any(|regex| regex.as_str() == compare_regex.as_str())
                        })
                    })
                    .progress_count(not_found.len() as u64)
                    .map(|key_url| {
                        let get_tags = m
                            .get(key_url)
                            .and_then(|get_key| self.find_any_method(get_key))
                            .and_then(|get_key| get_key.get("tags"))
                            .and_then(|get_key| get_key.as_sequence());

                        let tags_concatenated = get_tags
                            .and_then(|gt_contents| {
                                let tags_vec: Vec<_> = gt_contents.to_vec();
                                let tags: Vec<_> = tags_vec
                                    .par_iter()
                                    .map(|tag| tag.as_str().unwrap_or(NO_TAG))
                                    .collect();
                                if tags.is_empty() {
                                    None
                                } else {
                                    let tags_str = tags.join(",").as_str().to_string();
                                    Some(tags_str)
                                }
                            })
                            .unwrap_or(NO_TAG.to_string());
                        let get_params = self
                            .find_params(m.get(key_url))
                            .into_iter()
                            .collect::<Vec<(String, String)>>();
                        let params_info = get_params
                            .into_iter()
                            .map(|(s1, s2)| format!("{} -> {}", s1, s2).to_string())
                            .collect::<Vec<String>>()
                            .join("; ");

                        ReportRecord(
                            tags_concatenated, /*first_tag*/
                            String::from(key_url.as_str().unwrap_or(NO_URL)),
                            params_info,
                        )
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn find_any_method(&self, gk: &'a Value) -> Option<&'a Value> {
        let methods: Vec<_> = "get|post|put|patch|delete".split('|').collect();
        for method in methods {
            let try_method: Option<&'a Value> = gk.get(method);
            if try_method.is_some() {
                return try_method;
            }
        }
        None
    }

    fn transform_into_regex(&self, src: &str) -> Regex {
        let replace_regex = Regex::new(r"(\{\w*})").unwrap();
        Regex::new(format!("\"{}\"", replace_regex.replace_all(src, r"(\{\w*})")).as_ref()).unwrap()
    }

    fn find_params(&self, maybe_gk: Option<&'a Value>) -> Vec<(String, String)> {
        let methods = vec!["get", "post", "put", "patch", "delete"]; // "get|post|put|patch|delete".split('|').collect();
        if maybe_gk.is_none() {
            return vec![];
        };
        let gk = maybe_gk.unwrap();
        methods
            .into_iter()
            .flat_map(|method| {
                let curr_method = method.to_string();
                gk.get(method)
                    .into_iter()
                    .map(|method_val| {
                        let extrp = self.extract_parameters(method_val.get("parameters"));
                        (curr_method.clone(), extrp.join(", "))
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<(String, String)>>()
    }

    fn extract_parameters(&self, maybe_parameters: Option<&Value>) -> Vec<String> {
        maybe_parameters
            .iter()
            .flat_map(|parameters| match parameters {
                Value::Sequence(par_seq) => {
                    {
                        par_seq.iter().map(|param| {
                            match param {
                                Value::Mapping(param_val) => {
                                    let mut maybe_non_ref_param = param_val;
                                    if param_val.keys().collect_vec().first().is_some_and(|val| {
                                        *val == &Value::String(String::from("$ref"))
                                    }) {
                                        // if it is a $ref, search its path
                                        let ref_paths = self.split_ref(
                                            param_val.values().collect_vec()[0].as_str().unwrap(),
                                        );

                                        // as long as I didn't manage to write `fold` properly :/
                                        let mut last_val = self.all_yaml_data.as_ref();
                                        for ref_path in ref_paths {
                                            let maybe_last_val = last_val.get(ref_path);
                                            last_val = maybe_last_val.unwrap();
                                        }
                                        maybe_non_ref_param = last_val.as_mapping().unwrap();
                                    }
                                    let param_name = maybe_non_ref_param
                                        .get("name")
                                        .unwrap()
                                        .as_str()
                                        .unwrap_or_default();
                                    let param_in = maybe_non_ref_param
                                        .get("in")
                                        .unwrap()
                                        .as_str()
                                        .unwrap_or_default();
                                    let param_required = maybe_non_ref_param
                                        .get("required")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    let param_type = maybe_non_ref_param
                                        .get("schema")
                                        .and_then(|v| v.get("type"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("_");
                                    format!(
                                        "{}{param_name}: {param_type} (in: {param_in})",
                                        if param_required { "[reqd]" } else { "[optl]" }
                                    )
                                }
                                _ => String::new(),
                            }
                        })
                    }
                    .collect::<Vec<_>>()
                }
                _ => vec![],
            })
            .collect::<Vec<_>>()
    }

    /// ```
    /// let split_refs = split_ref("#/components/parameters/owner");
    /// assert_eq!(split_refs, ["components", "parameters", "owner"]);
    /// ```
    fn split_ref<'s>(&self, ref_str: &'s str) -> Vec<&'s str> {
        ref_str
            .strip_prefix("#/")
            .unwrap_or_default()
            .split('/')
            .collect()
    }
}

// mod tests {
//     #[test]
//     fn test_split_ref() {
//         let ref_str = "#/components/parameters/owner";
//         let split_refs: Vec<&str> = ref_str
//             .strip_prefix("#/")
//             .unwrap_or_default()
//             .split('/')
//             .collect();
//         assert_eq!(split_refs, ["components", "parameters", "owner"]);
//     }
// }
