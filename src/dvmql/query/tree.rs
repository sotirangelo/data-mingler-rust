use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::{dvmql::query::deserialization::DeserializedNode, transform::Transformation};

use super::deserialization::Query;

pub struct TreeNode {
    pub name: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub transformations: Vec<Transformation>,
    pub theta: Option<String>,
    pub output: bool,
}

impl TreeNode {
    fn new(
        name: String,
        label: String,
        children: Vec<TreeNode>,
        transformations: Vec<Transformation>,
        theta: Option<String>,
        output: bool,
    ) -> Self {
        Self {
            name,
            label,
            children,
            transformations,
            theta,
            output,
        }
    }
}

pub fn build_tree(query: Query) -> Result<TreeNode> {
    let mut deserialized_nodes_map: HashMap<String, DeserializedNode> = query
        .nodes
        .into_iter()
        .map(|node| (node.label.clone(), node))
        .collect();

    let root_node = deserialized_nodes_map
        .remove(&query.root_node)
        .with_context(|| {
            format!(
                "Incorrectly defined root node: {}. The <rootnode> tag and <label> of a node must match.",
                query.root_node.clone()
            )
        })?;

    let root_node = build_node(root_node, &mut deserialized_nodes_map)?;
    Ok(root_node)
}

fn build_node(
    node: DeserializedNode,
    nodes_map: &mut HashMap<String, DeserializedNode>,
) -> Result<TreeNode> {
    let mut tree_node = TreeNode::new(
        node.name,
        node.label,
        vec![],
        node.transformations,
        node.theta,
        node.output,
    );
    if !node.children.is_empty() {
        let children = node
            .children
            .iter()
            .map(|child_name| {
                let child = nodes_map
                    .remove(child_name).with_context(|| {
                        format!(
                            "Incorrectly defined child node: {}. The <label> of a node must match the <child> tag.",
                            child_name.clone()
                        )
                    })?;
                build_node(child, nodes_map)                            
            })
            .collect::<Result<Vec<TreeNode>>>()?;
        tree_node.children = children;
    }
    Ok(tree_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dvmql::query::deserialization::DeserializedNode;
    use crate::transform::aggregate::AggregationType;
    use crate::transform::Transformation;

    #[test]
    fn test_build_tree() {
        let query = Query {
            root_node: "X000".to_string(),
            nodes: vec![
                DeserializedNode {
                    name: "root".to_string(),
                    label: "X000".to_string(),
                    children: vec!["X001".to_string(), "X002".to_string()],
                    transformations: vec![Transformation::Aggregate(AggregationType::Min)],
                    theta: None,
                    output: false,
                },
                DeserializedNode {
                    name: "node1".to_string(),
                    label: "X001".to_string(),
                    children: vec![],
                    transformations: vec![],
                    theta: None,
                    output: false,
                },
                DeserializedNode {
                    name: "node2".to_string(),
                    label: "X002".to_string(),
                    children: vec!["X003".to_string()],
                    transformations: vec![],
                    theta: None,
                    output: false,
                },
                DeserializedNode {
                    name: "node3".to_string(),
                    label: "X003".to_string(),
                    children: vec![],
                    transformations: vec![],
                    theta: None,
                    output: true,
                },
            ],
        };
        let tree = build_tree(query).unwrap();
        assert_eq!(tree.name, "root");
        assert_eq!(tree.label, "X000");
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.transformations.len(), 1);
        assert_eq!(tree.theta, None);
        assert!(!tree.output);
        let node1 = tree.children.first().unwrap();
        assert_eq!(node1.name, "node1");
        assert_eq!(node1.label, "X001");
        assert_eq!(node1.children.len(), 0);
        assert_eq!(node1.transformations.len(), 0);
        assert_eq!(node1.theta, None);
        assert!(!node1.output);
        let node2 = tree.children.get(1).unwrap();
        assert_eq!(node2.name, "node2");
        assert_eq!(node2.label, "X002");
        assert_eq!(node2.children.len(), 1);
        assert_eq!(node2.transformations.len(), 0);
        assert_eq!(node2.theta, None);
        assert!(!node2.output);
        let node3 = node2.children.first().unwrap();
        assert_eq!(node3.name, "node3");
        assert_eq!(node3.label, "X003");
        assert_eq!(node3.children.len(), 0);
        assert_eq!(node3.transformations.len(), 0);
        assert_eq!(node3.theta, None);
        assert!(node3.output);
    }
}
