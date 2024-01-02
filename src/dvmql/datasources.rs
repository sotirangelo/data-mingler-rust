use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use super::helpers::read_xml_file;

pub fn load_datasources_xml(datasources_path: String) -> Result<HashMap<String, Datasource>> {
    let init_datasources: InitDatasources = read_xml_file(datasources_path)?;
    let res: HashMap<String, Datasource> = init_datasources
        .datasource
        .iter()
        .map(|init_ds| {
            let ds = match init_ds.ds_type.as_str() {
                "csv" => Datasource::Csv(Csv::from(init_ds)),
                "xml" => Datasource::Xml(Csv::from(init_ds)),
                "db" => Datasource::Database(Database::from(init_ds)),
                "excel" => Datasource::Excel(Excel::from(init_ds)),
                // TODO: Handle error
                &_ => panic!("Incorrect datasource type given: {}", init_ds.ds_type),
            };
            (init_ds.name.clone(), ds)
        })
        .collect();
    Ok(res)
}

#[derive(Deserialize, Debug, PartialEq)]
struct InitDatasources {
    datasource: Vec<InitDatasource>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct InitDatasource {
    #[serde(rename = "@type", default)]
    ds_type: String,
    id: u8,
    name: String,
    filename: Option<String>,
    path: Option<String>,
    sheet: Option<String>,
    delimiter: Option<String>,
    headings: Option<String>,
    system: Option<String>,
    connection: Option<String>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

impl InitDatasource {}

#[derive(Debug, PartialEq)]
pub enum Datasource {
    Csv(Csv),
    Xml(Csv),
    Excel(Excel),
    Database(Database),
}

#[derive(Debug, PartialEq)]
pub struct Csv {
    id: u8,
    name: String,
    filename: String,
    path: String,
    delimiter: String,
    headings: String,
}

impl From<&InitDatasource> for Csv {
    fn from(ds: &InitDatasource) -> Csv {
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
            headings: ds
                .headings
                .to_owned()
                .expect("CSV headings field should be defined"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Excel {
    id: u8,
    name: String,
    filename: String,
    path: String,
    sheet: String,
    headings: String,
}
impl From<&InitDatasource> for Excel {
    fn from(ds: &InitDatasource) -> Excel {
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
            headings: ds
                .headings
                .to_owned()
                .expect("Excel headings field should be defined"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Database {
    id: u8,
    name: String,
    system: String,
    connection: String,
    username: String,
    password: String,
    database: String,
}

impl From<&InitDatasource> for Database {
    fn from(ds: &InitDatasource) -> Database {
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

    fn get_init_datasources() -> InitDatasources {
        InitDatasources {
            datasource: vec![
                InitDatasource {
                    ds_type: String::from("csv"),
                    id: 1,
                    name: String::from("myCSV"),
                    filename: Some(String::from("file.csv")),
                    path: Some(String::from("/some-path/")),
                    delimiter: Some(String::from(",")),
                    headings: Some(String::from("yes")),
                    sheet: None,
                    system: None,
                    connection: None,
                    username: None,
                    password: None,
                    database: None,
                },
                InitDatasource {
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
                InitDatasource {
                    ds_type: String::from("xml"),
                    id: 3,
                    name: String::from("myCSV2"),
                    filename: Some(String::from("file.xml")),
                    path: Some(String::from("/some-path/")),
                    delimiter: Some(String::from(",")),
                    headings: Some(String::from("yes")),
                    sheet: None,
                    system: None,
                    connection: None,
                    username: None,
                    password: None,
                    database: None,
                },
                InitDatasource {
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
                delimiter: String::from(","),
                headings: String::from("yes"),
            }),
            Datasource::Excel(Excel {
                id: 2,
                name: String::from("myExcel"),
                filename: String::from("file.xlsx"),
                path: String::from("/some-path/"),
                sheet: String::from("Sheet1"),
                headings: String::from("no"),
            }),
            Datasource::Xml(Csv {
                id: 3,
                name: String::from("myCSV2"),
                filename: String::from("file.xml"),
                path: String::from("/some-path/"),
                delimiter: String::from(","),
                headings: String::from("yes"),
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
        let datasources: InitDatasources = read_xml_file(path).unwrap();
        assert_eq!(datasources, get_init_datasources());
    }

    #[test]
    fn test_load_datasources_xml() {
        let path = get_test_file_path();
        let datasources = load_datasources_xml(path).unwrap();
        assert_eq!(datasources, get_datasources());
    }
}
