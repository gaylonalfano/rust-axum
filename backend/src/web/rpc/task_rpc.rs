use crate::ctx::Ctx;
use crate::model::task::{Task, TaskBmc, TaskFilter, TaskForCreate, TaskForUpdate};
use crate::model::ModelManager;
use crate::web::rpc::params::{ParamsForCreate, ParamsForUpdate, ParamsIdOnly, ParamsList};
use crate::web::Result;

// NOTE: !! - Our design is as follows: Our ModelController (TaskBmc)
// will be very granular and will only return the id (TaskBmc::create -> Result<i64>).
// But this outward facing api to return the data back (Result<Task>).
// It's these functions that directly correspond to the JSON-RPC methods.
// Eg: /api/rpc => RpcRequest => RpcRequest.method => "list_tasks" => task_rpc::list_tasks();

pub async fn create_task(
    // NOTE: This is end of line for Ctx and MM, so we're consuming
    // them both but we could pass references if we wanted.
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {
    let ParamsForCreate { data } = params;

    let id = TaskBmc::create(&ctx, &mm, data).await?;
    let task = TaskBmc::get(&ctx, &mm, id).await?;

    Ok(task)
}

pub async fn list_tasks(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsList<TaskFilter>,
) -> Result<Vec<Task>> {
    let tasks = TaskBmc::list(&ctx, &mm, params.filters, params.list_options).await?;

    Ok(tasks)
}

pub async fn update_task(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {
    let ParamsForUpdate { id, data } = params;

    TaskBmc::update(&ctx, &mm, id, data).await?;

    let task = TaskBmc::get(&ctx, &mm, id).await?;

    Ok(task)
}

pub async fn delete_task(ctx: Ctx, mm: ModelManager, params: ParamsIdOnly) -> Result<Task> {
    let ParamsIdOnly { id } = params;

    let task = TaskBmc::get(&ctx, &mm, id).await?;
    TaskBmc::delete(&ctx, &mm, id).await?;

    Ok(task)
}
