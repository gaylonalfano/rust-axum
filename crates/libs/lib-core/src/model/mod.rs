//! Simplistic Model Layer
//! with mock-store layer)
//! Typically author prefers this pattern/structure:
//! Web (IPC)
//! Context -- Event
//! Model
//! Store
//! We're covering Web, Context and Model only
//! NOTE:
//! Model Layer
//! Design:
//! - The Model layer normalizes the application's data type
//!   structures and access.
//! - All application code data access must go through the Model layer.
//! - The `ModelManager` holds the internal states/resources
//!   needed by ModelControllers to access data.
//!   (e.g., db_pool, S3 client, redis client).
//! - Model Controllers (e.g., `TaskBmc`, `ProjectBmc`) implement
//!   CRUD and other data access methods on a given "entity"
//!   (e.g., `Task`, `Project`).
//!   (`Bmc` is short for Backend Model Controller).
//! - In frameworks like Axum, Tauri, `ModelManager` are typically used as App State.
//! - ModelManager are designed to be passed as an argument
//!   to all Model Controllers functions.

//! NOTE: We're adding a store layer as well. I believe
//! it'll go inside the Model Manager as a Model Controller,
//! but we'll see...

//! NOTE: This model module is the only one that touches the Db store
//! REF: https://youtu.be/-dMH9UiwKqg?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=1273

// region:       -- Modules

mod base;
mod error;
mod store;
pub mod task;
pub mod user;

// Re-export our model module Error and Result aliases
pub use self::error::{Error, Result};

use crate::model::store::{new_db_pool, Db};

// endregion:    -- Modules

// NOTE: Multiple States structure example (ModelManager/AppState)
// using FromRef trait (also a handy Axum macro)
// FromRef trait makes all properties (substates) a sub-state
// that can later inject. The cool thing is that you only
// need to use with_state(app_state) with the router and
// the handlers don't need to change! Nice. Easier to change.
// NOTE: Need to update cargo.toml Axum features=["macros"]
// #[derive(Clone, FromRef)]
// pub struct AppState {
//     // Sub-states go here...
//     pub mc: ModelController,
//     // redis: RedisConnector,
//     // s3: S3Bucket,
// }

// NOTE: !! U: With the baseline/monolithic approach, we coupled
// Axum into our core model layer using Axum's FromRef along
// with other stuff. With the multi-crate workspace approach,
// we want to decouple Axum from our lib-core, so we no longer
// import/depend on Axum inside our core.
// REF: https://youtu.be/zUxF0kvydJs
#[derive(Clone)]
pub struct ModelManager {
    // substates go here...
    // pub mc: ModelController,
    // redis: RedisConnector,
    // s3: S3Bucket,
    // etc.
    db: Db,
}

impl ModelManager {
    /// Constructor
    pub async fn new() -> Result<Self> {
        // NOTE: U: Removing this for now.
        // let mc = ModelController::new().await?;
        let db = new_db_pool().await?;

        // Ok(ModelManager { mc })
        Ok(ModelManager { db })
    }
    // NOTE: Only want to expose our Db (the db pool) ONLY
    // to the Model layer, and the 'new' accessible to other
    // modules such as main.rs.
    // E.g., If we tried from main.rs to use 'let db = mm.db()' it would fail!
    // NOTE: To restrict a function to ONLY sub-modules (i.e., store, error, model)
    // we use (in crate::model) syntax.
    // NOTE: What we end up with is our ModelManager::new() is accessible to
    // all other modules in the code base. Whereas ONLY the model layer
    // has access to the store (Db). Specifically, this returns the
    // sqlx db pool reference ONLY for the model layer.
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
