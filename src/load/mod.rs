use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_stream::stream;
use quick_xml::events::Event;
use tokio::{fs::File, sync::Mutex};
use tokio_stream::{Stream, StreamExt};

pub mod edges;

/// Enum representing the different types of datasources.
#[derive(Debug, PartialEq)]
pub enum Datasource {
    Csv(Csv),
    Xml(Xml),
    Excel(Excel),
    Database(Database),
}

/// CSV datasource
#[derive(Debug, PartialEq)]
pub struct Csv {
    pub id: u8,
    pub name: String,
    pub filename: String,
    pub path: String,
    pub delimiter: char,
    pub has_headers: bool,
}

/// XML datasource
#[derive(Debug, PartialEq)]
pub struct Xml {
    pub id: u8,
    pub name: String,
    pub filename: String,
    pub path: String,
}

/// Database datasource
#[derive(Debug, PartialEq)]
pub struct Database {
    pub id: u8,
    pub name: String,
    pub system: String,
    pub connection: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// Excel datasource
#[derive(Debug, PartialEq)]
pub struct Excel {
    pub id: u8,
    pub name: String,
    pub filename: String,
    pub path: String,
    pub sheet: String,
    pub has_headers: bool,
}

// TODO:
//
// - [ ] Clone reading xml from original data-mingler
// - [ ] Add functionality for reading from postgres DB
// - [ ] Integration testing for the `load` module: use dockertest crate
// - [ ] Make all streams return a Result of a custom record type

impl Csv {
    async fn read_async(
        &self,
        key_pos: usize,
        value_pos: usize,
    ) -> Result<impl Stream<Item = Result<(String, String)>>> {
        let file = File::open(&self.path).await?;
        let reader = csv_async::AsyncReaderBuilder::new()
            .has_headers(self.has_headers)
            .create_reader(file);
        Ok(reader
            .into_records()
            .map(move |x| -> Result<(String, String)> {
                let record = x?;
                let key = record.get(key_pos).unwrap().to_owned();
                let value = record.get(value_pos).unwrap().to_owned();
                Ok((key, value))
            }))
    }
}

impl Xml {
    async fn read_async(&self) -> Result<impl Stream<Item = Result<(String, String)>>> {
        let file = File::open(&self.path).await?;
        let reader = tokio::io::BufReader::new(file);
        let mut reader = quick_xml::Reader::from_reader(reader);
        let buf = Arc::new(Mutex::new(Vec::new()));
        let buf_shared = Arc::clone(&buf);
        let s = stream! {
            let mut key: Option<String> = None;
            let mut value: Option<String> = None;
            loop {
                match reader.read_event_into_async(&mut *buf_shared.lock().await).await {
                    Ok(Event::Eof) => break,
                    Ok(Event::Start(e)) =>
                    key = Some(String::from_utf8_lossy(e.name().local_name().into_inner()).into_owned()),
                    Ok(Event::Text(e)) =>
                        value = Some(e.unescape()?.into_owned()),
                    Ok(Event::End(_)) => {
                        if key.is_some() && value.is_some() {
                            yield Ok((key.unwrap(), value.unwrap()));
                        }
                        key = None;
                        value = None;
                    }
                    Err(e) => {
                        yield Err(anyhow!("Error reading event: {:?}", e));
                        break;
                    }
                    Ok(_) => continue,
                };
            }
        };
        buf.lock().await.clear();
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::pin_mut;
    use std::path::PathBuf;

    fn get_test_data_path(file: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path_str = "test_data/".to_string() + file;
        path.push(path_str);
        String::from(path.to_str().unwrap())
    }

    #[tokio::test]
    async fn test_csv_read_async_ignoring_headers() {
        let path = get_test_data_path("example_csv.csv");
        let csv = Csv {
            id: 1,
            name: "test".to_string(),
            filename: "test.csv".to_string(),
            path,
            delimiter: ',',
            has_headers: true,
        };
        let mut count = 0;
        let mut records = csv.read_async(1, 2).await.unwrap();
        let mut found_headers = false;
        while let Some(record) = records.next().await {
            count += 1;
            let record = record.unwrap();
            if record.0 == "firstname" && record.1 == "lastname" {
                found_headers = true;
            }
        }
        assert_eq!(count, 10);
        assert!(!found_headers);
    }

    #[tokio::test]
    async fn test_csv_read_async_with_headers() {
        let path = get_test_data_path("example_csv.csv");
        let csv = Csv {
            id: 1,
            name: "test".to_string(),
            filename: "test.csv".to_string(),
            path,
            delimiter: ',',
            has_headers: false,
        };
        let mut count = 0;
        let mut records = csv.read_async(1, 2).await.unwrap();
        let mut found_headers = false;
        while let Some(record) = records.next().await {
            count += 1;
            let record = record.unwrap();
            if record.0 == "firstname" && record.1 == "lastname" {
                found_headers = true;
            }
        }
        assert_eq!(count, 11);
        assert!(found_headers);
    }

    #[tokio::test]
    async fn test_xml_read_async() {
        let path = get_test_data_path("example_xml.xml");
        let xml = Xml {
            id: 1,
            name: "test".to_string(),
            filename: "test.xml".to_string(),
            path,
        };
        let keys = ["int_id", "name"];
        let mut count = 0;
        let records_stream = xml.read_async().await.unwrap();
        pin_mut!(records_stream);

        while let Some(record) = records_stream.next().await {
            count += 1;
            let record = record.unwrap();
            assert!(keys.contains(&record.0.as_str()));
        }
        assert_eq!(count, 8);
    }
}
