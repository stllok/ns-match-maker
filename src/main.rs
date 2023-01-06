use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use dotenvy::dotenv;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub mod docker;
pub mod web;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap())
        .with(tracing_subscriber::fmt::Layer::new())
        .try_init()?;

    Toplevel::new()
        .start("axum", web_api_server)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_secs(5))
        .await?;

    Ok(())
}

async fn web_api_server(subsys: SubsystemHandle) -> Result<()> {
    let app = web::get_route().await;
    tracing::info!("listening on bind addr");
    axum::Server::bind(&std::env::var("BIND_ADDR").unwrap().parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move {
            subsys.on_shutdown_requested().await;
        })
        .await?;
    Ok(())
}
