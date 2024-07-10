use assert_cmd::Command;

use predicates::prelude::predicate;
use testcontainers::{core::ContainerPort, ImageExt};
use testcontainers_modules::{neo4j::Neo4j, testcontainers::runners::AsyncRunner};

#[ignore]
#[tokio::test]
async fn it_loads_the_dvm() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let container = Neo4j::new()
        .with_version("latest")
        .with_user("neo4j")
        .with_password("12345678")
        .with_mapped_port(7687, ContainerPort::Tcp(7687))
        .start()
        .await?;

    let mut cmd = Command::cargo_bin("dvm-to-neo4j").unwrap();
    let assert = cmd
        .args(&[
            "-ddd",
            "--bolt-uri",
            format!(
                "bolt://{}:{}",
                container.get_host().await?,
                container.image().bolt_port_ipv4()?
            )
            .as_str(),
            "test_data/example_dvm.xml",
        ])
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("Starting DVM to Neo4j loader..."))
        .stdout(predicate::str::contains("Finished loading DVM to Neo4j"));

    // prepare neo4rs client
    let config = neo4rs::ConfigBuilder::new()
        .uri(format!(
            "bolt://{}:{}",
            container.get_host().await?,
            container.image().bolt_port_ipv4()?
        ))
        .user(container.image().user().expect("user is set"))
        .password(container.image().password().expect("password is set"))
        .build()?;

    // connect ot Neo4j
    let graph = neo4rs::Graph::connect(config).await?;

    // run a test query
    let mut rows = graph.execute(neo4rs::query("RETURN 1 + 1")).await?;
    while let Some(row) = rows.next().await? {
        let result: i64 = row.get("1 + 1").unwrap();
        assert_eq!(result, 2);
    }

    // see if the nodes are loaded
    let mut rows = graph
        .execute(neo4rs::query("MATCH (n) RETURN count(n)"))
        .await?;
    while let Some(row) = rows.next().await? {
        let result: i64 = row.get("count(n)").unwrap();
        assert_eq!(result, 3);
    }

    Ok(())
}
