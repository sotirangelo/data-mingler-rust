use anyhow::{Context, Result};
use quick_xml::de::from_reader;
use serde::de::DeserializeOwned;
use std::{fs::File, io::BufReader};

/// Helper function for reading an XML file and deserializing it into a struct.
pub fn read_xml_file<T: DeserializeOwned>(xml_file_path: &str) -> Result<T> {
    let file = File::open(xml_file_path)
        .with_context(|| format!("Failed to open file: {}", &xml_file_path))?;
    let reader = BufReader::new(file);
    from_reader(reader).with_context(|| format!("Failed to parse XML file: {}", &xml_file_path))
}
