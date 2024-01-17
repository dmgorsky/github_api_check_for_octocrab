use crate::swagger_parser::ReportRecord;
use itertools::Itertools;
use std::cmp::*;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct OutTsvWriter {
    file_name: String,
}

impl OutTsvWriter {
    pub(crate) fn new(out_file_name: String) -> Self {
        Self {
            file_name: out_file_name,
        }
    }

    pub fn write_to_csv(&self, records: &Vec<ReportRecord>) -> std::io::Result<usize> {
        let out_file = File::create(self.file_name.clone()).unwrap();
        let mut writer = BufWriter::new(out_file);
        writer.write_all(ReportRecord::tsv_header().as_bytes())?;
        for rec in records.iter().sorted_by(|a, b| a.0.cmp(&b.0)) {
            let rr: String = rec.to_string();
            writer.write_all(rr.as_bytes())?;
        }
        writer.flush()?;
        Ok(records.len())
    }
}
