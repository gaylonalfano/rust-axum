// FIXME: I believe this is for the Client-Side Logging (see log/mod.rs for Server Side)
use crate::ctx;
use crate::log::log_request;
use crate::web;
use crate::web::rpc::RpcInfo;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, to_value};
use tracing::debug;
use uuid::Uuid;

// Adding first layer (middleware)
// REF: Interesting relevant Axum details by Jon Gjengset: https://youtu.be/Wnb_n5YktO8?t=2273
// This is where the "magic" happens:
// REF: https://youtu.be/XZtlD_m59sM?t=4154
// U: Adding our log_request() helper for logging requests per line
// Thanks to Axum's Extractors, we can get all the needed info.
pub async fn mw_response_map(
    ctx: Option<ctx::Ctx>,
    uri: Uri,
    http_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");
    // Create a uuid to match our server errors to client errors
    let uuid = Uuid::new_v4();

    // -- Get RpcInfo
    let rpc_info = res.extensions().get::<RpcInfo>();

    // -- Get the eventual response error
    let service_error = res.extensions().get::<web::Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build a new response
    // Using as_ref() bc we're going to reuse this for server request logging
    // NOTE: U: After Serializing our ClientError enum (web::error.rs), we're
    // updating this to be more JSON RPC like.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            // U: After adding Serialize to ClientError to be more JSON RPC like.
            // We'll be extracting the tag="message" and content="detail"
            let client_error = serde_json::to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let detail = client_error.as_ref().and_then(|v| v.get("detail"));

            // U: Now we're making it more JSON RPC compliant with our structure
            // (id, error.{message,data{}})
            let client_error_body = json!({
                "id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
                "error": {
                    "message": message, // VariantName
                    "data": {
                        "req_uuid": uuid.to_string(),
                        "detail": detail // VariantData
                    }
                }
            });

            debug!("CLIENT ERROR BODY: {client_error_body}");

            // Build the new reponse from the client_error_body
            // NOTE:Recall we expanded into_response() to be a Axum Response
            // placeholder that takes the actual server error and
            // inserts it into the Response via extensions_mut()
            (*status_code, Json(client_error_body)).into_response()
        });

    // -- Build and log the server log line
    // NOTE: Server Reequests Log Line vs. Tracing
    // Tracing is adding warnings, info, debug, etc. inside your code so you can debug
    // Requests log line is one log line per request with error and other info.
    // You then can push to console.log() locally, and after deploying to the cloud
    // you can then use tools like CloudWatch and query with cloud-native tools.
    // NOTE: Option.unzip() gives us the Option<ClientError>
    let client_error = client_status_error.unzip().1;
    log_request(
        uuid,
        http_method,
        uri,
        rpc_info,
        ctx,
        service_error,
        client_error,
    )
    .await;

    debug!("\n");
    // NOTE:If we remove our quick_dev req_login(), we'll see the error uuids
    // match in the logs for both client and server errors. Neat!
    error_response.unwrap_or(res)
}
