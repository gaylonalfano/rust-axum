// NOTE: U: This is the result of the multi-crate upgrade,
// and splitting up the old/original web/rpc/mod.rs module
// to this file AND lib-rpc/src/lib.rs
use crate::web::mw_auth::CtxW;
use crate::web::Result;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use lib_rpc::{exec_rpc, RpcRequest};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

// region:    -- RPC Router & Handler
pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm) // Turns this Router into a Tower Service. See Jon's decrust.
}

/// RPC basic information holding the RPC request id and method for further logging
#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

// NOTE: U: Replacing Ctx with CtxW (wrapper) extractor since we need to implement
// external Traits (Ctx from lib-core & FromRequestParts from Axum) on the
// web layer's CtxW wrapper type. We can still access the real/inner Ctx using CtxW.0
async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: CtxW,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    // -- U: Extract the inner/real Ctx from our new CtxW wrapper
    let ctx = ctx.0;

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
    let rpc_method = rpc_req.method.clone();
    let rpc_id = rpc_req.id.clone();

    debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

    let result = exec_rpc(ctx, mm, rpc_req).await?;

    // Now that we have our JSON result, time to send our JSON response
    let body_response = json!({
    "id": rpc_id,
    "result": result
    });

    Ok(Json(body_response))
}
// endregion:    -- RPC Router & Handler
