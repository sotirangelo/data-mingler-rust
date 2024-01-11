mod dvmql;
mod load;
mod transform;

use std::collections::HashMap;

use anyhow::Result;
use async_recursion::async_recursion;

use clap::Parser;

use dvmql::{
    datasources::{self, Datasource},
    query::tree::TreeNode,
};
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
    }

    println!("Evaluating {}", &node.label);
    for child in &node.children {
        let mut result = graph
            .execute(
                query(QUERY)
                    .param("nodeA", child.name.as_str())
                    .param("nodeB", node.name.as_str()),
            )
            .await?;
        while let Some(r) = result.next().await? {
            let datasource: String = r.to::<Edge>().unwrap().datasource_name;
            let key: u32 = r.to::<Edge>().unwrap().key_pos;
            let value: u32 = r.to::<Edge>().unwrap().value_pos;
            println!("Datasource: {}, Key: {}, Value: {}", datasource, key, value);
            datasources.get(&datasource).unwrap_or_else(|| {
                panic!("Datasource {} not found in datasources list", &datasource)
            });
        }
        println!("Finished with child {}", &child.label);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    let _neo4j_graph = Graph::new("bolt://localhost:7687", "neo4j", "12345678").await?;
    assert!(_neo4j_graph.run(query("RETURN 1")).await.is_ok());

    let tree = load_query_xml(args.query_path)?;
    let _datasources = datasources::load_datasources_xml(args.datasources_path)?;

    dfs(&tree, &_neo4j_graph, &_datasources).await?;

    Ok(())
}
