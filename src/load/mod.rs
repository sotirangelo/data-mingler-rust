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
