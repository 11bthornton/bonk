use std::collections::HashMap;

use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::eval::eval_expr;

#[derive(Deserialize)]
struct EvalRequest {
    expr: String,
    #[serde(default)]
    context: Option<HashMap<String, JsonValue>>,
}

#[derive(Serialize)]
struct EvalResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn handle_eval(Json(req): Json<EvalRequest>) -> Json<EvalResponse> {
    match eval_expr(&req.expr, &req.context) {
        Ok(result) => Json(EvalResponse { result: Some(result), error: None }),
        Err(e) => Json(EvalResponse { result: None, error: Some(e) }),
    }
}

async fn handle_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn run() {
    let app = Router::new()
        .route("/", get(handle_index))
        .route("/eval", post(handle_eval));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("bonk server running at http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
