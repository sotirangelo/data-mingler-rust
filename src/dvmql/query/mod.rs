//! # Query
//!
//! This module containts deserialization logic for query XML files.

mod deserialization;
pub mod tree;

use anyhow::Result;
use log::{debug, info};

use self::{deserialization::Query, tree::TreeNode};
use crate::dvmql::helpers::read_xml_file;

/// Helper function for loading a query XML file and building the tree structure.
pub fn load_query_xml(query_path: &str) -> Result<TreeNode> {
    info!("Loading query from {}", query_path);
    let query: Query = read_xml_file(query_path)?;
    debug!("Deserialized query from XML {}", query_path);
    let tree = tree::build_tree(query)?;
    debug!("Built tree from query XML");
    Ok(tree)
}
