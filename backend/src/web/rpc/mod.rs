// region:       -- Modules

mod task_rpc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use std::sync::Arc;
use tracing::debug;

use crate::{
    ctx::Ctx,
    model::ModelManager,
    web::{
        rpc::task_rpc::{create_task, delete_task, list_tasks, update_task},
        Error, Result,
    },
};
// endregion:    -- Modules

// region:       -- RPC Types

/// JSON-RPC Request Body
// NOTE: At this level we'll just use a generic JSON Value type,
// but we'll do the actual parsing at the RPC routing level.
#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsIdOnly {
    id: i64,
}

/// RPC basic information holding the id and method for further logging
#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

// endregion:    -- RPC Types

// region:       -- RPC Router & Handler
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

// NOTE: Using proc macro to refactor our _rpc_handler to be
// more general and robust for additional entity types later on.
// REF: https://youtu.be/3cA_mk4vdWY?t=13160
macro_rules! exec_rpc_fn {
    // -- With Params (eg. create_task(ctx, mm, params))
    // NOTE: !! - Need to wrap with another layer of {} because the macro
    // will need to generate the code block {} in order for the
    // "match" statement in _rpc_handler to work. Specifically, the match will
    // expect a code block with {} because this logic isn't a one-liner,
    // hence the need to use/add {}s.
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        // NOTE: TIP: Use stringify!($rpc_fn) to get a string
        let rpc_fn_name = stringify!($rpc_fn);

        // Convert our rpc_params Option<Value> into a Result. This ensures
        // that we have params that are JSON Value type.
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        // We want a TaskForCreate type so we use serde_json::from_value()
        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;

        // We want this in the end, but we first need to get
        // RPC params into ParamsForCreate<TaskForCreate> type

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // -- Without Params (eg. list_tasks(ctx, mm))
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    // -- Create the RpcInfo to be set to the response.extensions
    // We'll later get/retrieve it for server login, request log line,
    // and errors we send back to the client.
    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    // -- Execute & Store RpcInfo in response
    let mut response = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    // NOTE: !! U: With Tower update, we now are inserting an Arc type into
    // the response extensions, so when we try to retrieve/extract this RpcInfo,
    // we actually have to extract the Arc type, not RpcInfo.
    response.extensions_mut().insert(Arc::new(rpc_info));

    response
}

/// Route based on RPC method and return a JSON result
async fn _rpc_handler(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Json<Value>> {
    // Destructure and rename inner props
    let RpcRequest {
        id: rpc_id,
        method: rpc_method,
        params: rpc_params,
    } = rpc_req;

    debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

    let result_json: Value = match rpc_method.as_str() {
        // -- Task RPC methods
        "create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
        "list_tasks" => {
            // NOTE: TIP: When first building a function, can add variables to debug,
            // and then remove afterwards: let r = list_tasks() + todo!()
            // NOTE: Using serde_json::to_value() returns a serde_json::Error,
            // but we want a web::Error instead, so we need to add a new
            // web::Error variant (SerdeJson(String)) and allow the conversion
            // by impl From<serde_json::Error> for Error {}
            exec_rpc_fn!(list_tasks, ctx, mm)
        }
        "update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
        "delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),

        // -- Fallback as Err.
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    // Now that we have our JSON result, time to send our JSON response
    let body_response = json!({
    "id": rpc_id,
    "result": result_json
    });

    Ok(Json(body_response))
}
// endregion:    -- RPC Router & Handler
