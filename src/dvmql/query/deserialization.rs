use std::str::FromStr;

use anyhow::Result;

use serde::de::Deserializer;
use serde::Deserialize;

use crate::dvmql::helpers::read_xml_file;
use crate::transform::aggregate::AggregationType;
use crate::transform::Transformation;

pub fn load_query_xml(query_path: String) -> Result<Query> {
    let query: Query = read_xml_file(query_path)?;
    Ok(query)
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Query {
    #[serde(rename = "rootnode")]
    pub root_node: String,
    #[serde(rename = "node")]
    pub nodes: Vec<DeserializedNode>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct DeserializedNode {
    #[serde(rename = "onnode")]
    pub name: String,
    pub label: String,
    #[serde(default, deserialize_with = "deserialize_children")]
    pub children: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_transformations")]
    pub transformations: Vec<Transformation>,
    #[serde(default, deserialize_with = "deserialize_theta")]
    pub theta: Option<String>,
    #[serde(default, deserialize_with = "deserialize_output")]
    pub output: bool,
}

fn deserialize_theta<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    Ok(Some(s))
}

fn deserialize_children<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(vec![]);
    }
    Ok(s.split(',').map(|part| part.trim().to_string()).collect())
}

fn deserialize_transformations<'de, D>(deserializer: D) -> Result<Vec<Transformation>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(vec![]);
    }
    let transformations: Vec<Transformation> = s
        .split(';')
        .map(|part| {
            let part: Vec<&str> = part.splitn(2, ':').collect();
            let transformation = part[0].trim();
            let args = part[1].trim();

            let transformation: Result<Transformation> = match transformation {
                "aggregate" => Ok(Transformation::Aggregate(
                    AggregationType::from_str(args).expect("should be valid aggregation"),
                )),
                "map" => Ok(Transformation::Map(args.to_string())),
                "filter" => Ok(Transformation::Filter(args.to_string())),
                &_ => panic!("Invalid transformation defined"), // TODO: Handle error
            };
            transformation.unwrap()
        })
        .collect();

    Ok(transformations)
}

fn deserialize_output<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(false);
    }
    return match s.to_lowercase().as_str() {
        "yes" | "true" => Ok(true),
        "no" | "false" => Ok(false),
        &_ => panic!("Invalid output given"), // TODO: Handle error
    };
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn get_expected_query() -> Query {
        Query {
            root_node: "X000".to_string(),
            nodes: vec![
                DeserializedNode {
                    label: "X000".to_string(),
                    name: "root_node".to_string(),
                    children: vec!["X001".to_string(), "X002".to_string()],
                    transformations: vec![],
                    theta: None,
                    output: false,
                },
                DeserializedNode {
                    label: "X001".to_string(),
                    name: "some_node".to_string(),
                    children: vec![],
                    transformations: vec![
                        Transformation::Filter("$X001$ > 5".to_string()),
                        Transformation::Aggregate(AggregationType::Sum),
                    ],
                    theta: None,
                    output: true,
                },
                DeserializedNode {
                    label: "X002".to_string(),
                    name: "some_other_node".to_string(),
                    children: vec![],
                    transformations: vec![],
                    theta: None,
                    output: false,
                },
            ],
        }
    }

    #[test]
    fn test_load_query_xml() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/example_query.xml");
        let path = path.to_str().unwrap().to_string();
        let expected_query = get_expected_query();
        assert_eq!(load_query_xml(path).unwrap(), expected_query)
    }
}
