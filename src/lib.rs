pub mod dvmql;
pub mod load;
pub mod transform;

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use neo4rs::{query, Graph};
use std::collections::HashMap;
use tracing::trace;

use dvmql::query::tree::TreeNode;
use load::Datasource;

use crate::load::edges::Edge;

const QUERY: &str = "MATCH (a:attribute{name: $nodeA})-[r:has]->(b:attribute{name: $nodeB}) RETURN r.datasource as datasource, r.query as query, r.key as key, r.value as value";

#[async_recursion]
pub async fn dfs(
    node: &TreeNode,
    graph: &Graph,
    datasources: &HashMap<String, Datasource>,
) -> Result<()> {
    for child in &node.children {
        dfs(child, graph, datasources).await?;

        let mut result = graph
            .execute(
                query(QUERY)
                    .param("nodeA", child.name.as_str())
                    .param("nodeB", node.name.as_str()),
            )
            .await?;
        while let Some(r) = result.next().await? {
            let datasource: String = r.to::<Edge>()?.datasource_name;
            let key: u32 = r.to::<Edge>()?.key_pos;
            let value: u32 = r.to::<Edge>()?.value_pos;
            let dt = datasources.get(&datasource).with_context(|| {
                format!("Datasource {} not found in datasources list", &datasource)
            })?;
            trace!("Edge {} => {}", &child.label, &node.label);
        }
    }
    Ok(())
}
