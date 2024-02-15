// region:       -- Modules

mod error;
mod params;
mod task_rpc;

pub use self::error::{Error, Result};
use params::*;

use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use task_rpc::{create_task, delete_task, list_tasks, update_task};

// endregion:    -- Modules

// region:       -- RPC Types

/// The raw JSON-RPC Request Body object. Foundation for RPC routing.
// NOTE: At this level we'll just use a generic JSON Value type,
// but we'll do the actual parsing at the RPC routing level.
#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

// endregion:    -- RPC Types

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
