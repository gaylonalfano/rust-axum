// NOTE: !! WIP My custom attempt for Solana token data via Birdeye API
// REF: https://docs.birdeye.so/docs/token-list

use crate::model::base::{self, DbBmc};
use crate::model::Result;
use crate::{ctx::Ctx, model::ModelManager};
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// region: -- Token Types
// NOTE: At a high level, structs are views on your db tables.
// We break up the structs between what we allow to be READ,
// and what we allow to be PUSHED. E.g., we don't want the API
// to change the creator of a task, or read certain properties.
// Therefore, we break up these structs to assist.
/// Sent back from model layer
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
// REF: https://docs.birdeye.so/docs/token-list
pub struct Token {
    pub id: i64,
    pub update_unix_time: i64,
    pub update_human_time: String,
    pub mint_address: String,
    pub decimals: i64,
    pub symbol: String,
    pub name: String,
    pub mc: f64,
    pub v24h_change_percent: f64,
    pub v24h_usd: f64,
}

/// Sent to model layer to update data structure
// U: Adding Fields to assist with building SQL statements
// U: Adding Default for new 'done: bool' property, so in
// our update() test, we can use ..Default::default()
#[derive(Fields, Default, Deserialize)]
pub struct TokenForCreate {
    // Don't want users via API to change the 'id' prop
    pub update_unix_time: i64,
    pub update_human_time: String,
    pub mint_address: String,
    pub decimals: i64,
    pub symbol: String,
    pub name: String,
    pub mc: f64,
    pub v24h_change_percent: f64,
    pub v24h_usd: f64,
}

/// Sent to model layer to update data structure
#[derive(Fields, Default, Deserialize)]
pub struct TokenForUpdate {
    pub update_unix_time: i64,
    pub update_human_time: String,
    pub mc: Option<f64>,
    pub v24h_change_percent: Option<f64>,
    pub v24h_usd: Option<f64>,
}

/// Filter by custom fields
// NOTE: modql traits in detail:
// - FilterNodes: ModQL trait to turn type into list of nodes for Sea Query
// - Deserialize: Allows type to have the '$' notation e.g., MongoDB
#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TokenFilter {
    // NOTE: TIP! Jeremy prefers to place the keys up top
    // with other props below with a line between.
    id: Option<OpValsInt64>,

    mint_address: Option<OpValsString>,
    v24h_change_percent: Option<OpValsInt64>,
    v24h_usd: Option<OpValsInt64>,
}
// endregion: -- Token Types

// region: -- TokenBmc
pub struct TokenBmc;

impl DbBmc for TokenBmc {
    const TABLE: &'static str = "token";
}

impl TokenBmc {
    // NOTE: Making create() very granular and efficient.
    // No need to return the full Task back. This also makes
    // our code reusable, since ctx and mm will be consistent for
    // other functions, but only the task type changes (task_c, task_u, etc.)
    // REF: https://youtu.be/3cA_mk4vdWY?t=3290
    pub async fn create(ctx: &Ctx, mm: &ModelManager, token_c: TokenForCreate) -> Result<i64> {
        // NOTE: Annotations can be inferred, but the compiler will see that
        // it's equivalent to: create::<TaskBmc, model::task::TaskForCreate>(ctx, mm, task_c)
        base::create::<Self, _>(ctx, mm, token_c).await

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

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Token> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    // NOTE: ModQL ListOptions - Offset, OrderBy, Limit
    pub async fn list(
        ctx: &Ctx,
        mm: &ModelManager,
        filters: Option<Vec<TokenFilter>>,
        list_options: Option<ListOptions>,
    ) -> Result<Vec<Token>> {
        // NOTE: TIP! Use a generic '_' to let compiler determine type (easier to change)
        base::list::<Self, _, _>(ctx, mm, filters, list_options).await

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
        token_u: TokenForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, token_u).await
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
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early dev & tests.

    use super::*;
    use crate::_dev_utils;
    // use crate::model::error::Error;

    use serde_json::json;
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
        // Q: How to yank/copy all the 'fx_' vars up to '='? I'd like to yank these vars
        // and then append below when creating the TokenForCreate {} test struct...
        // REF: https://stackoverflow.com/questions/23713617/vim-yank-all-matches-of-regex-group-into-register
        // :%s/fx_[^ ]* ---- highlights the var names but don't know how to then yank...
        // U: :let @a='' to empty register first, THEN: :%s/regex/\=setreg('A', submatch(0))/n

        let fx_update_unix_time = 1692203008;
        let fx_update_human_time = "2023-08-16T16:23:28";
        let fx_mint_address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let fx_decimals = 6;
        let fx_symbol = "USDC";
        let fx_name = "USD Coin";
        let fx_mc = 5034893047.819173;
        let fx_v24h_change_percent = 32.10423521982971;
        let fx_v24h_usd = 30582475.965653457;

        // -- Exec
        // NOTE: TIP: Use a debug print (println!("->> {task:?}")) at first
        // to ensure you get the expected output, and THEN use
        // the assert!() in the Check section.
        // Q: What's the difference between a Fixture and a Value?
        let token_c = TokenForCreate {
            update_unix_time: fx_update_unix_time,
            update_human_time: fx_update_human_time.to_string(),
            mint_address: fx_mint_address.to_string(),
            decimals: fx_decimals,
            symbol: fx_symbol.to_string(),
            name: fx_name.to_string(),
            mc: fx_mc,
            v24h_change_percent: fx_v24h_change_percent,
            v24h_usd: fx_v24h_usd,
        };
        let id = TokenBmc::create(&ctx, &mm, token_c).await?;

        // -- Check
        // let (title,): (String,) = sqlx::query_as("SELECT title FROM task WHERE id = $1")
        //     .bind(id)
        //     .fetch_one(mm.db())
        //     .await?;
        // println!("->> {title}");
        // assert_eq!(title, fx_title);
        let token = TokenBmc::get(&ctx, &mm, id).await?;
        assert_eq!(token.name, fx_name);

        // -- Clean
        // let count = sqlx::query("DELETE FROM task WHERE id = $1")
        //     .bind(id)
        //     .execute(mm.db())
        //     .await?
        //     .rows_affected();
        TokenBmc::delete(&ctx, &mm, id).await?;
        // assert_eq!(count, 1, "Did not delete 1 row?");

        Ok(())
    }

    // TODO: Revise tests for TOKEN
    // #[serial]
    // #[tokio::test]
    // async fn test_get_err_not_found() -> Result<()> {
    //     // -- Setup & Fixtures
    //     let mm = _dev_utils::init_test().await;
    //     let ctx = Ctx::root_ctx();
    //     let fx_id = 100;
    //
    //     // -- Exec
    //     let res = TaskBmc::get(&ctx, &mm, fx_id).await;
    //
    //     // -- Check
    //     // Q: How to assert we get the intended Error from our Result?
    //     // A: Use assert!(matches!(...)) macros!
    //     assert!(
    //         matches!(
    //             res,
    //             Err(crate::model::Error::EntityNotFound {
    //                 entity: "task",
    //                 id: 100 // Can't use variable in here!
    //             })
    //         ),
    //         "EntityNotFound not matching"
    //     );
    //
    //     Ok(())
    // }
    //
    // #[serial]
    // #[tokio::test]
    // async fn test_list_all_ok() -> Result<()> {
    //     // -- Setup & Fixtures
    //     let mm = _dev_utils::init_test().await;
    //     let ctx = Ctx::root_ctx();
    //     let fx_titles = &["test_list_all_ok-task 01", "test_list_all_ok-task 02"];
    //     _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;
    //
    //     // -- Exec
    //     let tasks = TaskBmc::list(&ctx, &mm, None, None).await?;
    //
    //     // -- Check
    //     // NOTE: To ensure we're checking against the correct tasks,
    //     // we're going to make sure the task titles match
    //     let tasks: Vec<Task> = tasks
    //         .into_iter()
    //         .filter(|t| t.title.starts_with("test_list_all_ok-task"))
    //         .collect();
    //     assert_eq!(tasks.len(), 2, "Number of seeded tasks");
    //
    //     // -- Clean
    //     for task in tasks.iter() {
    //         TaskBmc::delete(&ctx, &mm, task.id).await?;
    //     }
    //
    //     Ok(())
    // }
    //
    // #[serial]
    // #[tokio::test]
    // async fn test_list_by_filter_ok() -> Result<()> {
    //     // -- Setup & Fixtures
    //     let mm = _dev_utils::init_test().await;
    //     let ctx = Ctx::root_ctx();
    //     let fx_titles = &[
    //         "test_list_by_filter_ok-task 01.a",
    //         "test_list_by_filter_ok-task 01.b",
    //         "test_list_by_filter_ok-task 02.a",
    //         "test_list_by_filter_ok-task 02.b",
    //         "test_list_by_filter_ok-task 03",
    //     ];
    //     _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;
    //
    //     // -- Exec
    //     // NOTE: Lots of Modql filter operations
    //     // REF: https://youtu.be/-dMH9UiwKqg?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=1975
    //     let filters: Vec<TaskFilter> = serde_json::from_value(json!([
    //         {
    //             // "title": "test_list_by_filter_ok-task 01.a"
    //             "title": {
    //                 "$endsWith": ".a",
    //                 // "$contains": "01",
    //                 "$containsAny": ["01", "02"],
    //             }
    //         },
    //         {
    //         "title": {"$contains": "03"}
    //         }
    //     ]))?;
    //     let list_options: ListOptions = serde_json::from_value(json!({
    //         // "limit": 1,
    //         "order_bys": "!id",
    //     }))?;
    //
    //     let tasks = TaskBmc::list(&ctx, &mm, Some(filters), Some(list_options)).await?;
    //
    //     // -- Check
    //     // NOTE: TIP! When first writing tests, we can remove the check
    //     // and just use a simple debug print. Later add check.
    //     // NOTE: To ensure we're checking against the correct tasks,
    //     // we're going to make sure the task titles match
    //     assert_eq!(tasks.len(), 3);
    //     assert!(tasks[0].title.ends_with("03"));
    //     assert!(tasks[1].title.ends_with("02.a"));
    //     assert!(tasks[2].title.ends_with("01.a"));
    //
    //     // -- Clean
    //     let tasks = TaskBmc::list(
    //         &ctx,
    //         &mm,
    //         Some(serde_json::from_value(json!([{
    //             "title": {"$startsWith": "test_list_by_filter_ok"}
    //         }]))?),
    //         None,
    //     )
    //     .await?;
    //     assert_eq!(tasks.len(), 5);
    //
    //     for task in tasks.iter() {
    //         TaskBmc::delete(&ctx, &mm, task.id).await?;
    //     }
    //
    //     Ok(())
    // }
    //
    // #[serial]
    // #[tokio::test]
    // async fn test_update_ok() -> Result<()> {
    //     // -- Setup & Fixtures
    //     let mm = _dev_utils::init_test().await;
    //     let ctx = Ctx::root_ctx();
    //     let fx_title = "test_update_ok - task 01";
    //     let fx_title_new = "test_update_ok - task 01 - new";
    //     let fx_task = _dev_utils::seed_tasks(&ctx, &mm, &[fx_title])
    //         .await?
    //         .remove(0);
    //
    //     // -- Exec
    //     TaskBmc::update(
    //         &ctx,
    //         &mm,
    //         fx_task.id,
    //         TaskForUpdate {
    //             title: Some(fx_title_new.to_string()),
    //             // U: Added 'Default' trait to TaskForUpdate, so 'done: bool'
    //             ..Default::default()
    //         },
    //     )
    //     .await?;
    //
    //     // -- Check
    //     let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
    //     assert_eq!(task.title, fx_title_new);
    //
    //     // -- Clean
    //     TaskBmc::delete(&ctx, &mm, task.id).await?;
    //
    //     Ok(())
    // }
    //
    // #[serial]
    // #[tokio::test]
    // async fn test_delete_err_not_found() -> Result<()> {
    //     // -- Setup & Fixtures
    //     let mm = _dev_utils::init_test().await;
    //     let ctx = Ctx::root_ctx();
    //     let fx_id = 100;
    //
    //     // -- Exec
    //     let res = TaskBmc::delete(&ctx, &mm, fx_id).await;
    //
    //     assert!(
    //         matches!(
    //             res,
    //             Err(crate::model::Error::EntityNotFound {
    //                 entity: "task",
    //                 id: 100
    //             })
    //         ),
    //         "EntityNotFound not matching"
    //     );
    //
    //     Ok(())
    // }
}
// endregion: -- Tests
