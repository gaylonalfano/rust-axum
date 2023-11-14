//!
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

use axum::extract::FromRef;

// use crate::model::ModelController;
use crate::{Error, Result};

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

#[derive(Clone, FromRef)]
pub struct ModelManager {
    // substates go here...
    // pub mc: ModelController,
    // redis: RedisConnector,
    // s3: S3Bucket,
    // etc.
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        // FIXME: U: Removing this for now.
        // let mc = ModelController::new().await?;

        // Ok(ModelManager { mc })
        Ok(ModelManager {})
    }

    // pub(in crate::model) fn db(&self) -> &Db {...}
}
