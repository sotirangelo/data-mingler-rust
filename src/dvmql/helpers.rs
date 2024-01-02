use std::{fs::File, io::BufReader};

use anyhow::{Context, Result};
use quick_xml::de::from_reader;
use serde::de::DeserializeOwned;

pub fn read_xml_file<T: DeserializeOwned>(xml_file_path: String) -> Result<T> {
    let file = File::open(&xml_file_path)
        .with_context(|| format!("Failed to open file: {}", &xml_file_path))?;
    let reader = BufReader::new(file);
    from_reader(reader).with_context(|| format!("Failed to parse XML file: {}", &xml_file_path))
}
