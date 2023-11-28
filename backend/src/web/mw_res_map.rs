use crate::ctx;
use crate::log::log_request;
use crate::web;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
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
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");
    // Create a uuid to match our server errors to client errors
    let uuid = Uuid::new_v4();

    // - Get the eventual response error
    let service_error = res.extensions().get::<web::Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // - If client error, build a new response
    // Using as_ref() bc we're going to reuse this for server request login
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(), // Thanks to strum_macros
                    "req_uuid": uuid.to_string(),
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
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    debug!("\n");
    // NOTE:If we remove our quick_dev req_login(), we'll see the error uuids
    // match in the logs for both client and server errors. Neat!
    error_response.unwrap_or(res)
}
