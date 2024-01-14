//! # Query
//!
//! This module containts deserialization logic for query XML files.

mod deserialization;
pub mod tree;

use anyhow::Result;

use self::{deserialization::Query, tree::TreeNode};
use crate::dvmql::helpers::read_xml_file;

/// Helper function for loading a query XML file and building the tree structure.
pub fn load_query_xml(query_path: String) -> Result<TreeNode> {
    let query: Query = read_xml_file(query_path)?;
    let tree = tree::build_tree(query)?;
    Ok(tree)
}
