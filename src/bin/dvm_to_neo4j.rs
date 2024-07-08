use anyhow::{anyhow, Result};
use clap::Parser;
use neo4rs::{query, Graph};
use quick_xml::events::Event;
use tokio::fs::File;
use tracing::{debug, info, trace, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    dvm_file_path: String,
    #[arg(long, default_value_t = String::from("bolt://localhost:7687"))]
    bolt_uri: String,
    #[arg(long)]
    use_existing_graph: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logger
    let log_level = match args.debug {
        1 => Level::INFO,
        2 => Level::DEBUG,
        3 => Level::TRACE,
        _ => Level::ERROR,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    info!("Starting DVM to Neo4j loader...");

    // Initialize Neo4j graph
    let neo4j = Graph::new(args.bolt_uri, "neo4j", "12345678").await?;
    assert!(neo4j.run(query("RETURN 1")).await.is_ok());
    debug!("Connected to Neo4j");

    if !args.use_existing_graph {
        // Clear the graph
        neo4j.run(query("MATCH (n) DETACH DELETE n")).await?;
        debug!("Cleared the graph");
    }

    load_dvm_to_neo4j(&neo4j, &args.dvm_file_path).await?;

    info!("Finished loading DVM to Neo4j");
    Ok(())
}

async fn load_dvm_to_neo4j(graph: &Graph, dvm_file_path: &str) -> Result<()> {
    info!("Reading from DVM file \"{}\"", dvm_file_path);
    let file = File::open(dvm_file_path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut reader = quick_xml::Reader::from_reader(reader);

    let mut buf = Vec::new();
    loop {
        let mut node_a_name = String::new();
        let mut node_a_description = String::new();
        let mut node_b_name = String::new();
        let mut node_b_description = String::new();
        let mut datasource_name = String::new();
        let mut query_string = String::new();
        let mut pos1 = String::new();
        let mut pos2 = String::new();

        let mut in_headnode = false;
        let mut in_tailnode = false;
        let mut in_datasource = false;
        let mut in_query = false;
        let mut in_key = false;
        let mut in_value = false;

        loop {
            match reader.read_event_into_async(&mut buf).await {
                Ok(Event::Start(ref e)) => match e.name().local_name().as_ref() {
                    b"headnode" => in_headnode = true,
                    b"tailnode" => in_tailnode = true,
                    b"datasource" => in_datasource = true,
                    b"query" => in_query = true,
                    b"key" => in_key = true,
                    b"value" => in_value = true,
                    b"name" | b"description" | b"edge" | b"edges" => (),
                    _ => return Err(anyhow!("Unexpected start tag: {:?}", e))?,
                },
                Ok(Event::End(ref e)) => match e.name().local_name().as_ref() {
                    b"headnode" => in_headnode = false,
                    b"tailnode" => in_tailnode = false,
                    b"datasource" => in_datasource = false,
                    b"query" => in_query = false,
                    b"key" => in_key = false,
                    b"value" => in_value = false,
                    b"name" | b"description" | b"edges" => (),
                    b"edge" => break,
                    _ => return Err(anyhow!("Unexpected end tag: {:?}", e))?,
                },
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.trim().to_owned();

                    if text.is_empty() {
                        continue;
                    }

                    if in_headnode {
                        if node_a_name.is_empty() {
                            node_a_name.clone_from(&text);
                        } else {
                            node_a_description.clone_from(&text);
                        }
                    } else if in_tailnode {
                        if node_b_name.is_empty() {
                            node_b_name.clone_from(&text);
                        } else {
                            node_b_description.clone_from(&text);
                        }
                    } else if in_datasource {
                        datasource_name.clone_from(&text);
                    } else if in_query {
                        query_string.clone_from(&text);
                    } else if in_key {
                        pos1.clone_from(&text);
                    } else if in_value {
                        pos2.clone_from(&text);
                    } else {
                        return Err(anyhow!("Unexpected text: {:?}", text))?;
                    }
                }
                Ok(Event::Eof) => return Ok(()),
                Err(e) => return Err(anyhow!("Error reading event: {:?}", e)),
                _ => (),
            }

            buf.clear();
        }

        debug!("Storing edge: \"{}\" -> \"{}\"", node_a_name, node_b_name);
        let store_node_a = store_node_in_neo4j(graph, &node_a_name, &node_a_description);
        let store_node_b = store_node_in_neo4j(graph, &node_b_name, &node_b_description);
        tokio::try_join!(store_node_a, store_node_b).map_err(|e| {
            anyhow!(format!(
                "Error storing while edge \"{}\" -> \"{}\": {}",
                node_a_name, node_b_name, e
            ))
        })?;

        graph.run(query(
            "MATCH (a:attribute), (b:attribute) \
            WHERE a.name = $nodeA_name AND b.name = $nodeB_name \
            CREATE (a)-[:has {datasource: $datasource_name, query: $query_string, key: $pos1, value: $pos2, selected: 'false'}]->(b)")
            .param("nodeA_name", node_a_name.clone())
            .param("nodeB_name", node_b_name.clone())
            .param("datasource_name", datasource_name)
            .param("query_string", query_string)
            .param("pos1", pos1)
            .param("pos2", pos2)
        ).await?;

        let node_a_query = graph.run(
            query(
                "MATCH (n:attribute {name: $nodeA_name})-[:has]->(b) \
            WITH n, count(b) AS cnt \
            WHERE cnt > 1 \
            SET n:primary",
            )
            .param("nodeA_name", node_a_name),
        );

        let node_b_query = graph.run(
            query(
                "MATCH (n:attribute {name: $nodeB_name})-[:has]->(b) \
            WITH n, count(b) AS cnt \
            WHERE cnt > 1 \
            SET n:primary",
            )
            .param("nodeB_name", node_b_name),
        );

        tokio::try_join!(node_a_query, node_b_query).map_err(|e| anyhow!(e))?;
    }
}

async fn store_node_in_neo4j(neo4j: &Graph, node_name: &str, node_description: &str) -> Result<()> {
    let mut exists_query_result = neo4j
        .execute(
            query("MATCH (n:attribute {name: $nodeName}) RETURN n.name AS name")
                .param("nodeName", node_name),
        )
        .await?;
    if (exists_query_result.next().await?).is_none() {
        trace!("Node does not exist, creating node {}", node_name);
        neo4j
            .run(
                query("CREATE (n:attribute {name: $nodeName, description: $nodeDescription})")
                    .param("nodeName", node_name)
                    .param("nodeDescription", node_description),
            )
            .await?;
    } else {
        trace!(
            "Node already exists, skipping creation of \"{}\"",
            node_name
        );
    }
    Ok(())
}
