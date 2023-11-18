use crate::model::{Error, Result};
use crate::{ctx::Ctx, model::model_manager::ModelManager};
use serde::{Deserialize, Serialize};
use sqlb::{HasFields, SqlBuilder, Whereable};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

// NOTE: ! - Explanation of this design approach. Two video snippets:
// TL;DR - We can use functions + Generics + Trait bounds to implement
// shared implementation between a base MC and specialized (task) MC.
// REF: https://youtu.be/3cA_mk4vdWY?t=6012
// REF: https://youtu.be/3cA_mk4vdWY?t=6146
// NOTE: We're refactoring out the common CRUD parts
// to be more general across various entities (not just Tasks).
// We're going to use Traits, Generics and Macros to implement
// this shared impl between all Model Controllers.
// REF: https://youtu.be/3cA_mk4vdWY?t=4739
pub trait DbBmc {
    const TABLE: &'static str;
}

// NOTE: TIP: sqlb::HasFields allows us to extract the fields on data argument (E)
// name and value, so that we can inject it without knowing the concrete type passed.
// Again, this is the model::base layer, so we want it to be generic for all entity types.
pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;

    Ok(id)
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
    E: HasFields,
{
    let db = mm.db();

    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .and_where("id", "=", id)
        .fetch_optional(db)
        .await? // Fail if db error
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names())
        .order_by("id") // "!id" for desc order.
        .fetch_all(db)
        .await?; // Fail if db error

    Ok(entities)
}

// NOTE: Our Bmc API is going to be more general, so we're going to return void ().
// However, our web API can be more convenient and return something else
// REF: https://youtu.be/3cA_mk4vdWY?t=5801
pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();

    let count = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
