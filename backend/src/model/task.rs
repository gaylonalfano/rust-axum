use crate::model::{Error, Result};
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

    pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        let db = mm.db();

        let task: Task = sqlx::query_as("SELECT * FROM task WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await? // Fail if db error
            .ok_or(Error::EntityNotFound { entity: "task", id })?;

        Ok(task)
    }

    pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        let db = mm.db();

        let count = sqlx::query("DELETE FROM task WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();
        // assert_eq!(count, 1, "Did not delete 1 row?");

        if count == 0 {
            return Err(Error::EntityNotFound { entity: "task", id });
        }

        Ok(())
    }
}
// endregion: -- TaskBmc

// region: -- Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use crate::_dev_utils;

    use super::*;
    use anyhow::Result;
    use serial_test::serial;

    // NOTE: Convention with some variations
    // E.g., test_create_ok, test_create_ok_simple,
    // test_create_ok_double_create, test_create_err_duplicate, etc.
    // NOTE: Tests in 'cargo test' run in parallel, so it's tricky
    // to synchronize them, especially if they have EXTERNAL resources
    // To help with this, we use crate 'serial', so now each fn
    // that has #[serial] annotation will run serially.
    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        // -- Setup & Fixtures
        // NOTE: Ideally our _dev_utils mod could return a ModelManager
        // for us to work with. We don't want init_dev() to do this,
        // since its called from main.rs. However, we can create a new
        // function specific for our test environment.
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        // -- Exec
        // Q: What's the difference between a Fixture and a Value?
        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        // -- Check
        // let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
        //     .bind(id)
        //     .fetch_one(mm.db())
        //     .await?;
        // println!("->> {title}");
        // assert_eq!(title, fx_title);
        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        // -- Clean
        // let count = sqlx::query("DELETE FROM task WHERE id = $1")
        //     .bind(id)
        //     .execute(mm.db())
        //     .await?
        //     .rows_affected();
        TaskBmc::delete(&ctx, &mm, id).await?;
        // assert_eq!(count, 1, "Did not delete 1 row?");

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        // -- Exec
        let res = TaskBmc::get(&ctx, &mm, fx_id).await;
        // Q: How to assert we get the intended Error from our Result?
        // A: Use assert!(matches!(...)) macros!
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100 // Can't use variable in here!
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        // -- Exec
        let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound not matching"
        );

        Ok(())
    }
}
// endregion: -- Tests
