use crate::model::base::{self, DbBmc};
use crate::model::{Error, Result};
use crate::{ctx::Ctx, model::model_manager::ModelManager};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;

// region: -- Task Types
// NOTE: At a high level, structs are views on your db tables.
// We break up the structs between what we allow to be READ,
// and what we allow to be PUSHED. E.g., we don't want the API
// to change the creator of a task, or read certain properties.
// Therefore, we break up these structs to assist.
/// Sent back from model layer
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    // -- sqlb example:
    // #[field(skip)] // sqlb::Fields
    // pub something_else: String,
    //
    // #[field(name = "description")] // sqlb::Fields
    // #[sqlx(rename = "description")] // sqlx::FromRow
    // pub desc: String,
}

/// Sent to model layer to update data structure
// U: Adding Fields to assist with building SQL statements
#[derive(Fields, Deserialize)]
pub struct TaskForCreate {
    // Don't want users via API to change the 'id' prop
    pub title: String,
}

/// Sent to model layer to update data structure
#[derive(Fields, Deserialize)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}
// endregion: -- Task Types

// region: -- TaskBmc
pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}

impl TaskBmc {
    // NOTE: Making create() very granular and efficient.
    // No need to return the full Task back. This also makes
    // our code reusable, since ctx and mm will be consistent for
    // other functions, but only the task type changes (task_c, task_u, etc.)
    // REF: https://youtu.be/3cA_mk4vdWY?t=3290
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        // NOTE: Annotations can be inferred, but the compiler will see that
        // it's equivalent to: create::<TaskBmc, model::task::TaskForCreate>(ctx, mm, task_c)
        base::create::<Self, _>(ctx, mm, task_c).await

        // -- BEFORE base layer:
        // let db = mm.db();
        //
        // // NOTE: TIP: Simple guard against SQL injection is to use parameters
        // // like ($1, $2) in your statements instead of raw values.
        // // NOTE: Use '_' generic but Rust will infer the type (i.e., 'Postgres')
        // let (id,) =
        //     sqlx::query_as::<_, (i64,)>("INSERT INTO task (title) values ($1) returning id")
        //         .bind(task_c.title)
        //         .fetch_one(db)
        //         .await?;
        //
        // Ok(id)
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, mm).await

        // -- BEFORE base layer:
        // let db = mm.db();
        //
        // let tasks: Vec<Task> = sqlx::query_as("SELECT * FROM task ORDER BY id")
        //     .fetch_all(db)
        //     .await?;
        //
        // Ok(tasks)
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: TaskForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task_u).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await

        // -- BEFORE base layer:
        // let db = mm.db();
        //
        // let count = sqlx::query("DELETE FROM task WHERE id = $1")
        //     .bind(id)
        //     .execute(db)
        //     .await?
        //     .rows_affected();
        // // assert_eq!(count, 1, "Did not delete 1 row?");
        //
        // if count == 0 {
        //     return Err(Error::EntityNotFound { entity: "task", id });
        // }
        //
        // Ok(())
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
        // NOTE: TIP: Use a debug print (println!("->> {task:?}")) at first
        // to ensure you get the expected output, and THEN use
        // the assert!() in the Check section.
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
    async fn test_list_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &["test_list_ok-task 01", "test_list_ok-task 02"];
        _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        // -- Exec
        let tasks = TaskBmc::list(&ctx, &mm).await?;

        // -- Check
        // NOTE: To ensure we're checking against the correct tasks,
        // we're going to make sure the task titles match
        let tasks: Vec<Task> = tasks
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_ok-task"))
            .collect();
        assert_eq!(tasks.len(), 2, "Number of seeded tasks");

        // -- Clean
        for task in tasks.iter() {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_update_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_update_ok - task 01";
        let fx_title_new = "test_update_ok - task 01 - new";
        let fx_task = _dev_utils::seed_tasks(&ctx, &mm, &[fx_title])
            .await?
            .remove(0);

        // -- Exec
        TaskBmc::update(
            &ctx,
            &mm,
            fx_task.id,
            TaskForUpdate {
                title: Some(fx_title_new.to_string()),
            },
        )
        .await?;

        // -- Check
        let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
        assert_eq!(task.title, fx_title_new);

        // -- Clean
        TaskBmc::delete(&ctx, &mm, task.id).await?;

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
