use std::{io::BufReader, fs::File, sync::Arc};

use docker_api::{Docker, models::ContainerSummary, opts::{ContainerFilter, ContainerListOpts}};
use once_cell::sync::Lazy;
use serde_json::Value;
use anyhow::Result;

static DOCKER: Lazy<Docker> = Lazy::new(|| {
    Docker::new("unix:///var/run/docker.sock").expect("Cannot get connection from docker daemon")
});
static SERVER_CONFIG: Lazy<Value> = Lazy::new(|| {
    serde_json::from_reader(BufReader::new(
        File::open("config.json").expect("Unable to open config.json"),
    ))
    .expect("Unable to decode config,json to json")
});


pub async fn acquire() -> Result<(Option<u32>, Option<u32>)> {
    let servers = Arc::new(containers().await?);

    let auth_port = (SERVER_CONFIG["StartMasterPort"].as_u64().unwrap() as u32
        ..SERVER_CONFIG["EndMasterPort"].as_u64().unwrap() as u32)
        .find(|&p| {
            !servers.iter().any(|server| {
                server
                    .labels
                    .as_ref()
                    .and_then(|server| {
                        server
                            .get("UsedAuthPort")
                            .and_then(|server| server.parse::<u32>().ok())
                    })
                    .eq(&Some(p))
            })
        });

    let port = (SERVER_CONFIG["StartServerPort"].as_u64().unwrap() as u32
        ..SERVER_CONFIG["EndServerPort"].as_u64().unwrap() as u32)
        .find(|&p| {
            !servers.iter().any(|server| {
                server
                    .labels
                    .as_ref()
                    .and_then(|server| {
                        server
                            .get("UsedPort")
                            .and_then(|server| server.parse::<u32>().ok())
                    })
                    .eq(&Some(p))
            })
        });

    Ok((port, auth_port))
}

pub async fn containers(
) -> Result<Vec<ContainerSummary>, docker_api::Error> {
    // default fillter
    let fillter = vec![ContainerFilter::Label("UsedBy".into(), "NSCN".into())];

    DOCKER
        .containers()
        .list(&ContainerListOpts::builder().filter(fillter).build())
        .await
}