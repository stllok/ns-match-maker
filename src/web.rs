use crate::docker;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

pub async fn get_route() -> Router {
    Router::new()
        .route("/", get(|| async { "Service online!" }))
        .route("/server/:data", post(create_server).delete(remove_server))
}

async fn create_server(Path(paths): Path<HashMap<String, String>>) -> Json<Value> {
    match docker::create_server(paths.get("data").unwrap_or(&"".into()).to_string()).await {
        Ok(token) => {
            info!("Receive create_server requests: {token}");
            Json(json!({
                "success": true,
                "token": token
            }))
        }
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}

async fn remove_server(Path(paths): Path<HashMap<String, String>>) -> Json<Value> {
    match docker::remove_container_via_id(paths.get("data").unwrap_or(&"".into()).to_string()).await
    {
        Ok(msg) => {
            info!("Receive remove_server requests: {msg}");
            Json(json!({
                "success": true,
                "message": msg
            }))
        }
        Err(err) => Json(json!({
            "success": false,
            "error": err.to_string()
        })),
    }
}
