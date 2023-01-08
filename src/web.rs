use crate::docker;
use axum::{
    routing::{get, post},
    Form, Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

pub async fn get_route() -> Router {
    Router::new()
        .route("/", get(|| async { "Service online!" }))
        .route("/server", post(create_server).delete(remove_server))
}

async fn create_server(Form(form): Form<HashMap<String, String>>) -> Json<Value> {
    match form.get("id") {
        Some(id) => {
            match docker::create_server(id.as_str(), form.get("playlist").unwrap_or(&"".into()))
                .await
            {
                Ok(( port, auth_port)) => {
                    Json(json!({
                        "success": true,
                        "port": port,
                        "auth_port": auth_port,
                    }))
                }
                Err(err) => Json(json!({
                    "success": false,
                    "error": err.to_string()
                })),
            }
        }
        None => todo!(),
    }
}

async fn remove_server(Form(form): Form<HashMap<String, String>>) -> Json<Value> {
    match docker::remove_container_via_id(form.get("id").unwrap_or(&"".into())).await {
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
