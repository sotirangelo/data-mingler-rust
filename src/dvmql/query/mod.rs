mod deserialization;
pub mod tree;

use anyhow::Result;

use self::tree::TreeNode;

pub fn load_query_xml(query_path: String) -> Result<TreeNode> {
    let query = deserialization::load_query_xml(query_path)?;
    let tree = tree::build_tree(query)?;
    Ok(tree)
}
