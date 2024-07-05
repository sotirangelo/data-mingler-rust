use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use neo4rs::{query, Graph};

use data_mingler_rust::{dfs, dvmql::datasources, dvmql::query::load_query_xml};

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
