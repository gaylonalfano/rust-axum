// NOTE: !! This is for our Server-Side Request Log
use crate::web::routes_rpc::RpcInfo;
use crate::web::{self, ClientError};
use crate::Result;
use axum::http::{Method, Uri};
use lib_core::ctx::Ctx;
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use uuid::Uuid;

// NOTE: Goal of this is we'll call this inside our
// response mapper to our RequestLogLine
// NOTE: Axum's Extractors help us get all this info
// for our main_response_mapper
pub async fn log_request(
    uuid: Uuid,
    http_method: Method,
    uri: Uri,
    rpc_info: Option<&RpcInfo>,
    ctx: Option<Ctx>,
    service_error: Option<&web::Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    // Timestamp hack for now (should be UTC iso8601)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let service_error_type = service_error.map(|se| se.as_ref().to_string());
    let service_error_data = serde_json::to_value(service_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    // Create the RequestLogLine
    let request_log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),

        user_id: ctx.map(|c| c.user_id()),

        http_path: uri.to_string(),
        http_method: http_method.to_string(),

        rpc_id: rpc_info.and_then(|rpc| rpc.id.as_ref().map(|id| id.to_string())),
        rpc_method: rpc_info.map(|rpc| rpc.method.to_string()),

        client_error_type: client_error.map(|e| e.as_ref().to_string()),
        error_type: service_error_type,
        error_data: service_error_data,
    };

    debug!("REQUEST LOG LINE: \n{}", json!(request_log_line));

    // TODO: Send to cloud-watch service
    Ok(())
}

// NOTE: Important to make line as flat as possible for querying.
// We serialize to get one line JSON
// skip_serializing_none so Option::None does not get serialized.
// Option::Some(T) gets serialized.
#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,      // uuid string formatted
    timestamp: String, // (should be iso8601)
    // -- User and context attributes
    user_id: Option<i64>,

    // -- http request attributes
    http_path: String,
    http_method: String,

    // -- RPC Info
    rpc_id: Option<String>,
    rpc_method: Option<String>,

    // -- Errors attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
