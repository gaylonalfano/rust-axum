use crate::model::{Error, Result};
use crate::{ctx::Ctx, model::model_manager::ModelManager};
use serde::{Deserialize, Serialize};
use sqlb::HasFields;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

// NOTE: We're refactoring out the common CRUD parts
// to be more general across various entities (not just Tasks).
// We're going to use Traits, Generics and Macros to implement
// this shared impl between all Model Controllers.
// REF: https://youtu.be/3cA_mk4vdWY?t=4739
pub trait DbBmc {
    const TABLE: &'static str;
}

// NOTE: We can use functions as shared implementation.
// We can use Generics to pass types information into functions
// using Trait bounds.
// NOTE: U: Adding 'sqlb' crate to help build our SQL statements.
// The challenge is that an update() or create() would require
// us to pass some 'data', but sqlx only has FromRow, not ToRow.
// Therefore, sqlb sits on top of sqlx to help in this case.
// REF: https://youtu.be/3cA_mk4vdWY?t=5298
/// MC = Model Controller generic
/// E = Entity
pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();

    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entity: E = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(db)
        .await? // Fail if db error
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}
