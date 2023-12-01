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
use tracing::debug;

use crate::{
    ctx::Ctx,
    model::ModelManager,
    web::{
        rpc::task_rpc::{create_task, list_tasks},
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
// endregion:    -- RPC Types

// region:       -- RPC Router & Handler
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    _rpc_handler(ctx, mm, rpc_req).await.into_response()
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
        "create_task" => {
            // Convert our rpc_params Option<Value> into a Result. This ensures
            // that we have params that are JSON Value type.
            let params = rpc_params.ok_or(Error::RpcMissingParams {
                rpc_method: "create_task".to_string(),
            })?;
            // We want a TaskForCreate type so we use serde_json::from_value()
            let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
                rpc_method: "create_task".to_string(),
            })?;

            // We want this in the end, but we first need to get
            // RPC params into ParamsForCreate<TaskForCreate> type
            create_task(ctx, mm, params).await.map(to_value)??
        }
        "list_tasks" => {
            // NOTE: TIP: When first building a function, can add variables to debug,
            // and then remove afterwards: let r = list_tasks() + todo!()
            // NOTE: Using serde_json::to_value() returns a serde_json::Error,
            // but we want a web::Error instead, so we need to add a new
            // web::Error variant (SerdeJson(String)) and allow the conversion
            // by impl From<serde_json::Error> for Error {}
            list_tasks(ctx, mm).await.map(to_value)??
        }
        "update_task" => todo!(),
        "delete_task" => todo!(),

        // -- Fallback as Err.
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    // Now that we have our JSON result, time to send our JSON response
    let body_response =
        json!({
        "id": rpc_id,
        "result": result_json
        });

    Ok(Json(body_response))
}
// endregion:    -- RPC Router & Handler
