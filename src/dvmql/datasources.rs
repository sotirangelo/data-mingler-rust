//! # Datasources
//!
//! This module containts deserialization logic for the datasources XML file.

use anyhow::Result;
use log::{debug, info, trace};
use serde::Deserialize;
use std::collections::HashMap;

use crate::load::{Csv, Database, Datasource, Excel, Xml};

use super::helpers::read_xml_file;

/// Helper function for loading and deserializing the datasources XML file
pub fn load_datasources_xml(datasources_path: String) -> Result<HashMap<String, Datasource>> {
    info!("Loading datasources from {}", datasources_path);
    let init_datasources: DeserializedDatasources = read_xml_file(&datasources_path)?;
    debug!("Deserialized datasources from XML {}", datasources_path);
    let res: HashMap<String, Datasource> = init_datasources
        .datasource
        .iter()
        .map(|init_ds| {
            let ds = match init_ds.ds_type.as_str() {
                "csv" => Datasource::Csv(Csv::from(init_ds)),
                "xml" => Datasource::Xml(Xml::from(init_ds)),
                "db" => Datasource::Database(Database::from(init_ds)),
                "excel" => Datasource::Excel(Excel::from(init_ds)),
                // TODO: Handle error
                &_ => panic!("Incorrect datasource type given: {}", init_ds.ds_type),
            };
            trace!(
                "Collecting {} datasource: {}",
                init_ds.ds_type.to_uppercase(),
                init_ds.name
            );
            (init_ds.name.clone(), ds)
        })
        .collect();
    debug!("Built collection of datasources from datasources XML");
    Ok(res)
}

/// Intermediate representation of the deserialized datasources XML file.
#[derive(Deserialize, Debug, PartialEq)]
struct DeserializedDatasources {
    datasource: Vec<DeserializedDatasource>,
}

/// Intermediate representation of a datasource in the deserialized datasources XML file.
#[derive(Deserialize, Debug, PartialEq)]
struct DeserializedDatasource {
    #[serde(rename = "@type", default)]
    ds_type: String,
    id: u8,
    name: String,
    filename: Option<String>,
    path: Option<String>,
    sheet: Option<String>,
    delimiter: Option<char>,
    headings: Option<String>,
    system: Option<String>,
    connection: Option<String>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

const HEADINGS_HAYSTACK: [&str; 4] = ["yes", "true", "y", "1"];

impl From<&DeserializedDatasource> for Csv {
    fn from(ds: &DeserializedDatasource) -> Csv {
        Csv {
            id: ds.id,
            name: ds.name.to_owned(),
            filename: ds
                .filename
                .to_owned()
                .expect("CSV filename field should be defined"),
            path: ds
                .path
                .to_owned()
                .expect("CSV path field should be defined"),
            delimiter: ds
                .delimiter
                .to_owned()
                .expect("CSV delimiter field should be defined"),
            has_headers: HEADINGS_HAYSTACK.contains(
                &ds.headings
                    .to_owned()
                    .expect("CSV headings should be defined")
                    .as_str(),
            ),
        }
    }
}

impl From<&DeserializedDatasource> for Xml {
    fn from(ds: &DeserializedDatasource) -> Xml {
        Xml {
            id: ds.id,
            name: ds.name.to_owned(),
            filename: ds
                .filename
                .to_owned()
                .expect("CSV filename field should be defined"),
            path: ds
                .path
                .to_owned()
                .expect("CSV path field should be defined"),
        }
    }
}

impl From<&DeserializedDatasource> for Excel {
    fn from(ds: &DeserializedDatasource) -> Excel {
        Excel {
            id: ds.id,
            name: ds.name.to_owned(),
            filename: ds
                .filename
                .to_owned()
                .expect("Excel filename field should be defined"),
            path: ds
                .path
                .to_owned()
                .expect("Excel path field should be defined"),
            sheet: ds
                .sheet
                .to_owned()
                .expect("Excel sheet field should be defined"),
            has_headers: HEADINGS_HAYSTACK.contains(
                &ds.headings
                    .to_owned()
                    .expect("Excel headings should be defined")
                    .as_str(),
            ),
        }
    }
}

impl From<&DeserializedDatasource> for Database {
    fn from(ds: &DeserializedDatasource) -> Database {
        Database {
            id: ds.id,
            name: ds.name.to_owned(),
            system: ds
                .system
                .to_owned()
                .expect("Database system field should be defined"),
            connection: ds
                .connection
                .to_owned()
                .expect("Database connection field should be defined"),
            username: ds
                .username
                .to_owned()
                .expect("Database username field should be defined"),
            password: ds
                .password
                .to_owned()
                .expect("Database password field should be defined"),
            database: ds
                .database
                .to_owned()
                .expect("Database database field should be defined"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn get_init_datasources() -> DeserializedDatasources {
        DeserializedDatasources {
            datasource: vec![
                DeserializedDatasource {
                    ds_type: String::from("csv"),
                    id: 1,
                    name: String::from("myCSV"),
                    filename: Some(String::from("file.csv")),
                    path: Some(String::from("/some-path/")),
                    delimiter: Some(','),
                    headings: Some(String::from("yes")),
                    sheet: None,
                    system: None,
                    connection: None,
                    username: None,
                    password: None,
                    database: None,
                },
                DeserializedDatasource {
                    ds_type: String::from("excel"),
                    id: 2,
                    name: String::from("myExcel"),
                    filename: Some(String::from("file.xlsx")),
                    path: Some(String::from("/some-path/")),
                    sheet: Some(String::from("Sheet1")),
                    headings: Some(String::from("no")),
                    delimiter: None,
                    system: None,
                    connection: None,
                    username: None,
                    password: None,
                    database: None,
                },
                DeserializedDatasource {
                    ds_type: String::from("xml"),
                    id: 3,
                    name: String::from("myCSV2"),
                    filename: Some(String::from("file.xml")),
                    path: Some(String::from("/some-path/")),
                    delimiter: Some(','),
                    headings: Some(String::from("yes")),
                    sheet: None,
                    system: None,
                    connection: None,
                    username: None,
                    password: None,
                    database: None,
                },
                DeserializedDatasource {
                    ds_type: String::from("db"),
                    id: 4,
                    name: String::from("sqlDb"),
                    system: Some(String::from("postgresql")),
                    connection: Some(String::from("localhost:5432")),
                    username: Some(String::from("bdms")),
                    password: Some(String::from("mysecretpassword")),
                    database: Some(String::from("postgres")),
                    filename: None,
                    path: None,
                    delimiter: None,
                    headings: None,
                    sheet: None,
                },
            ],
        }
    }

    fn get_datasources() -> HashMap<String, Datasource> {
        let vector = vec![
            Datasource::Csv(Csv {
                id: 1,
                name: String::from("myCSV"),
                filename: String::from("file.csv"),
                path: String::from("/some-path/"),
                delimiter: ',',
                has_headers: true,
            }),
            Datasource::Excel(Excel {
                id: 2,
                name: String::from("myExcel"),
                filename: String::from("file.xlsx"),
                path: String::from("/some-path/"),
                sheet: String::from("Sheet1"),
                has_headers: false,
            }),
            Datasource::Xml(Xml {
                id: 3,
                name: String::from("myCSV2"),
                filename: String::from("file.xml"),
                path: String::from("/some-path/"),
            }),
            Datasource::Database(Database {
                id: 4,
                name: String::from("sqlDb"),
                system: String::from("postgresql"),
                connection: String::from("localhost:5432"),
                username: String::from("bdms"),
                password: String::from("mysecretpassword"),
                database: String::from("postgres"),
            }),
        ];
        let mut ds_map: HashMap<String, Datasource> = HashMap::new();
        for ds in vector {
            let name = match &ds {
                Datasource::Csv(csv) => csv.name.clone(),
                Datasource::Excel(excel) => excel.name.clone(),
                Datasource::Database(db) => db.name.clone(),
                Datasource::Xml(xml) => xml.name.clone(),
            };
            ds_map.insert(name, ds);
        }
        ds_map
    }

    fn get_test_file_path() -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/example_datasources.xml");
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_loading_initial_datasource_structs() {
        let path = get_test_file_path();
        let datasources: DeserializedDatasources = read_xml_file(&path).unwrap();
        assert_eq!(datasources, get_init_datasources());
    }

    #[test]
    fn test_load_datasources_xml() {
        let path = get_test_file_path();
        let datasources = load_datasources_xml(path).unwrap();
        assert_eq!(datasources, get_datasources());
    }
}
