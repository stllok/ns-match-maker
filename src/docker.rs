use std::{fs::File, io::BufReader, sync::Arc};

use anyhow::{anyhow, Result};
use docker_api::{
    models::ContainerSummary,
    opts::{
        ContainerCreateOpts, ContainerFilter, ContainerListOpts, ContainerRemoveOpts, PublishPort,
    },
    Docker, Container,
};
use once_cell::sync::Lazy;
use serde_json::Value;

static DOCKER: Lazy<Docker> = Lazy::new(|| {
    Docker::new("unix:///var/run/docker.sock").expect("Cannot get connection from docker daemon")
});
static SERVER_CONFIG: Lazy<Value> = Lazy::new(|| {
    serde_json::from_reader(BufReader::new(
        File::open("config.json").expect("Unable to open config.json"),
    ))
    .expect("Unable to decode config.json to json")
});

pub async fn create_server(gamemode: String) -> Result<(String, u32, u32)> {
    match acquire().await {
        Ok((Some(port), Some(auth_port))) => {
            let labels = [
                ("UsedBy", "NSCN"),
                ("NSCNGamemode", &gamemode),
                ("UsedPort", &port.to_string()),
                ("UsedAuthPort", &auth_port.to_string()),
            ];

            let env = [
                format!(
                    "NS_EXTRA_ARGUMENTS={} {}",
                    SERVER_CONFIG["globalArgument"].as_str().unwrap(),
                    SERVER_CONFIG["gameserver"][&gamemode]["Argument"]
                        .as_str()
                        .ok_or_else(|| anyhow!("gamemode not found"))?,
                ),
                format!(
                    "NS_MASTERSERVER_URL={}",
                    SERVER_CONFIG["masterserver"].as_str().unwrap()
                ),
                format!(
                    "NS_SERVER_DESC={}",
                    SERVER_CONFIG["description"].as_str().unwrap()
                ),
                format!("NS_PORT={port}"),
                format!("NS_PORT_AUTH={auth_port}"),
                "NS_INSECURE=0".into(),
                "NS_RETURN_TO_LOBBY=0".into(),
            ];

            let volumes = [
                format!(
                    "{}/content:/mnt/titanfall:ro",
                    SERVER_CONFIG["content_dir"].as_str().unwrap()
                ),
                format!(
                    "{}{}:/mnt/mods:ro",
                    SERVER_CONFIG["content_dir"].as_str().unwrap(),
                    SERVER_CONFIG[&gamemode]["mods_dir"]
                        .as_str()
                        .unwrap_or("/default")
                ),
            ];

            // create container (most using config.json)
            let container = DOCKER
                .containers()
                .create(
                    &ContainerCreateOpts::builder()
                        .image(SERVER_CONFIG["image"].as_str().expect("IMAGE not provided"))
                        .memory_swap(-1)
                        .auto_remove(true)
                        .tty(true)
                        .labels(labels)
                        .network_mode("bridge")
                        .expose(PublishPort::udp(port), port)
                        .expose(PublishPort::tcp(auth_port), auth_port)
                        .env(env)
                        .volumes(volumes)
                        .build(),
                )
                .await?;

            container.start().await?;

            Ok((container.id().to_string(), port, auth_port))
        }
        _ => Err(anyhow!("Unable to assign port or auth_port")),
    }
}

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

async fn containers() -> Result<Vec<ContainerSummary>, docker_api::Error> {
    // default fillter
    let fillter = vec![ContainerFilter::Label("UsedBy".into(), "NSCN".into())];

    DOCKER
        .containers()
        .list(&ContainerListOpts::builder().filter(fillter).build())
        .await
}

pub async fn remove_container_via_id(id: String) -> Result<String> {
    Container::new(DOCKER.clone(), id)
        .remove(&ContainerRemoveOpts::builder().force(true).build())
        .await
        .map_err(|err| anyhow!(err))
}
