mod dvmql;
mod load;
mod transform;

use std::collections::HashMap;

use anyhow::{Context, Result};
use async_recursion::async_recursion;

use clap::Parser;

use dvmql::{datasources, query::tree::TreeNode};
use env_logger::Builder;
use load::Datasource;
use log::{trace, LevelFilter};
use neo4rs::{query, Graph};

use crate::{dvmql::query::load_query_xml, load::edges::Edge};

// TODO: Add arguments for neo4j db
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    datasources_path: String,
    query_path: String,
    #[arg(short, long, default_value_t = String::from("NONE"))]
    output: String,
    #[arg(short, long, default_value_t = String::from("ALL"))]
    mode: String,
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

const QUERY: &str = "MATCH (a:attribute{name: $nodeA})-[r:has]->(b:attribute{name: $nodeB}) RETURN r.datasource as datasource, r.query as query, r.key as key, r.value as value";

#[async_recursion]
async fn dfs(
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

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logger
    let log_level = match args.debug {
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3 => LevelFilter::Trace,
        _ => LevelFilter::Error,
    };
    Builder::new().filter_level(log_level).init();

    // Initialize Neo4j graph
    let neo4j = Graph::new("bolt://localhost:7687", "neo4j", "12345678").await?;
    assert!(neo4j.run(query("RETURN 1")).await.is_ok());

    // Load query & datasources
    let tree = load_query_xml(args.query_path)?;
    let datasources = datasources::load_datasources_xml(args.datasources_path)?;

    // Execute query
    dfs(&tree, &neo4j, &datasources).await?;

    Ok(())
}
