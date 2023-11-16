use crate::model::Result;
use crate::{ctx::Ctx, model::model_manager::ModelManager};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// region: -- Task Types
// NOTE: At a high level, structs are views on your db tables.
// We break up the structs between what we allow to be READ,
// and what we allow to be PUSHED. E.g., we don't want the API
// to change the creator of a task, or read certain properties.
// Therefore, we break up these structs to assist.
/// Sent back from model layer
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

/// Sent to model layer to update data structure
#[derive(Deserialize)]
pub struct TaskForCreate {
    // Don't want users via API to change the 'id' prop
    pub title: String,
}

/// Sent to model layer to update data structure
#[derive(Deserialize)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}
// endregion: -- Task Types

// region: -- TaskBmc
pub struct TaskBmc;

impl TaskBmc {
    // NOTE: Making create() very granular and efficient.
    // No need to return the full Task back. This also makes
    // our code reusable, since ctx and mm will be consistent for
    // other functions, but only the task type changes (task_c, task_u, etc.)
    // REF: https://youtu.be/3cA_mk4vdWY?t=3290
    pub async fn create(_ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        let db = mm.db();

        // NOTE: TIP: Simple guard against SQL injection is to use parameters
        // like ($1, $2) in your statements instead of raw values.
        // NOTE: Use '_' generic but Rust will infer the type (i.e., 'Postgres')
        let (id,) =
            sqlx::query_as::<_, (i64,)>("INSERT INTO task (title) values ($1) returning id")
                .bind(task_c.title)
                .fetch_one(db)
                .await?;

        Ok(id)
    }
}
// endregion: -- TaskBmc
