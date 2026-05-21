use axum::Json;
use serde_json::json;

pub fn health_response(service: &'static str) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": service }))
}
