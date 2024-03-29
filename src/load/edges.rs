use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, PartialEq)]
pub struct Edge {
    #[serde(rename = "datasource")]
    pub datasource_name: String,
    #[serde(rename = "key", deserialize_with = "deserialize_number_from_string")]
    pub key_pos: u32,
    #[serde(rename = "value", deserialize_with = "deserialize_number_from_string")]
    pub value_pos: u32,
    pub query: Option<String>,
}

pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}
