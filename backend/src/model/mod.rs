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

mod error;
mod model_manager;
mod store;

// Re-export our module Error and Result aliases
// pub use self::error::{Error, Result};
pub use error::*;
pub use model_manager::*;

use crate::model::store::{new_db_pool, Db};
