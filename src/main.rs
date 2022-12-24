use anyhow::Result;
use dotenvy::dotenv;
use tracing_subscriber::{EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

pub mod docker;
pub mod web;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();


    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap())
        .with(tracing_subscriber::fmt::Layer::new())
        .try_init()?;

    Ok(())
}