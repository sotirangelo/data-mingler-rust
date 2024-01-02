use anyhow::Result;

use serde::Deserialize;

use super::helpers::read_xml_file;

pub fn load_query_xml(query_path: String) -> Result<Query> {
    let mut query: Query = read_xml_file(query_path)?;
    query
        .node
        .iter_mut()
        .for_each(|node| node.clear_empty_strings());
    Ok(query)
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Query {
    root_node: String,
    node: Vec<InitNode>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct InitNode {
    #[serde(rename = "onnode")]
    name: String,
    label: String,
    children: Option<String>,
    transformations: Option<String>,
    theta: Option<String>,
    output: Option<String>,
}

impl InitNode {
    fn clear_empty_strings(&mut self) {
        if let Some(children) = &mut self.children {
            if children.is_empty() {
                self.children = None;
            }
        }
        if let Some(transformations) = &mut self.transformations {
            if transformations.is_empty() {
                self.transformations = None;
            }
        }
        if let Some(theta) = &mut self.theta {
            if theta.is_empty() {
                self.theta = None;
            }
        }
        if let Some(output) = &mut self.output {
            if output.is_empty() {
                self.output = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn get_expected_query() -> Query {
        Query {
            root_node: "X000".to_string(),
            node: vec![
                InitNode {
                    label: "X000".to_string(),
                    name: "root_node".to_string(),
                    children: Some("X001".to_string()),
                    transformations: None,
                    theta: None,
                    output: None,
                },
                InitNode {
                    label: "X001".to_string(),
                    name: "some_node".to_string(),
                    children: None,
                    transformations: Some("filter: $X001$ > 5;aggregate:sum".to_string()),
                    theta: None,
                    output: Some("yes".to_string()),
                },
                InitNode {
                    label: "X002".to_string(),
                    name: "some_other_node".to_string(),
                    children: None,
                    transformations: None,
                    theta: None,
                    output: None,
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

    #[test]
    fn test_node_field_clearing() {
        let mut node = InitNode {
            label: "X001".to_string(),
            name: "some_node".to_string(),
            children: Some("".to_string()),
            transformations: Some("aggregate:sum".to_string()),
            theta: Some("".to_string()),
            output: Some("yes".to_string()),
        };
        node.clear_empty_strings();
        assert_eq!(
            node,
            InitNode {
                label: "X001".to_string(),
                name: "some_node".to_string(),
                children: None,
                transformations: Some("aggregate:sum".to_string()),
                theta: None,
                output: Some("yes".to_string()),
            }
        )
    }
}
