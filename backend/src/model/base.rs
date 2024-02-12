use crate::model::{Error, Result};
use crate::{ctx::Ctx, model::model_manager::ModelManager};
use modql::field::HasFields;
use modql::filter::{FilterGroup, FilterGroups, ListOptions};
use modql::SIden;
use sea_query::{
    Condition, ConditionalStatement, Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef,
};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
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

const LIST_LIMIT_DEFAULT: i64 = 300;
const LIST_LIMIT_MAX: i64 = 1000;

// NOTE: This enum is like a Sea Query table and columns
// REF: https://youtu.be/-dMH9UiwKqg?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=561
#[derive(Iden)]
pub enum CommonIden {
    Id,
}

pub trait DbBmc {
    const TABLE: &'static str;

    // Helper fn to get a sea query table reference
    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

pub fn finalize_list_options(list_options: Option<ListOptions>) -> Result<ListOptions> {
    // -- When Some, validate limit
    if let Some(mut list_options) = list_options {
        // Validate the limit
        if let Some(limit) = list_options.limit {
            if limit > LIST_LIMIT_MAX {
                return Err(Error::ListLimitOverMax {
                    max: LIST_LIMIT_MAX,
                    actual: limit,
                });
            }
        }
        // Set to default is no limit provided
        else {
            list_options.limit = Some(LIST_LIMIT_DEFAULT);
        }
        Ok(list_options)
    }
    // -- When None, return default limit
    else {
        Ok(ListOptions {
            limit: Some(LIST_LIMIT_DEFAULT),
            offset: None,
            order_bys: Some("id".into()),
        })
    }
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

    // -- Prep data & Extract fields (name / sea-query value expression)
    let fields = data.not_none_fields();
    // Reformat our fields into a sea-query format for building our query
    // REF: https://youtu.be/-dMH9UiwKqg?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=458
    let (columns, sea_values) = fields.for_sea_insert();

    // -- Build the query w/ sea-query
    // NOTE: The builder pattern in sea-query is a "Ref Mut" pattern
    // Check out my own builder-pattern repo for details!
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIden::Id]));

    // -- Exec query w/ SQLx
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
        .fetch_one(db)
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
    // U: Old. Now we have Sea Query + ModQL
    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);

    // -- Build the query w/ sea-query
    // NOTE: The builder pattern in sea-query is a "Ref Mut" pattern
    // Check out my own builder-pattern repo for details!
    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::field_column_refs())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query w/ SQLx
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

// NOTE: U: Adding filtering ability w/ modql::filter::FilterGroups and Sea Query
// FilterNodes are set up in groups, and groups can be composed together.
// This makes the monomorphization of first(?) allows us to pass any types as
// filters that implements the FilterNodes, which impls Into<FilterGroups>.
// REF: https://youtu.be/-dMH9UiwKqg?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=1611
pub async fn list<MC, E, F>(
    _ctx: &Ctx,
    mm: &ModelManager,
    filters: Option<F>,
    list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
    F: Into<FilterGroups>,
{
    let db = mm.db();
    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);

    // -- Build the query w/ sea-query
    // NOTE: The builder pattern in sea-query is a "Ref Mut" pattern
    // Check out my own builder-pattern repo for details!
    let mut query = Query::select();
    query.from(MC::table_ref()).columns(E::field_column_refs());

    // Add condtion from filter
    if let Some(filters) = filters {
        let filters: FilterGroups = filters.into();
        // NOTE: Had to add a new ModqlIntoSeaQuery Error enum variant for filtering (see
        // model/error.rs) for details
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // List options
    // NOTE:U: TIP! - The problem of doing an 'if let Some(list_options) is that our
    // call to list_options.apply_to_sea_query() will only run IF we
    // pass in actual list options. This leaves our SELECT statement unbounded!
    // Better is to ALWAYS call this apply_to_sea_query() with some sort of default.
    let list_options = finalize_list_options(list_options)?;
    list_options.apply_to_sea_query(&mut query);

    // -- Exec query w/ SQLx
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_all(db)
        .await?;

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

    // -- Prep data
    let fields = data.not_none_fields();
    // Reformat our fields into a sea-query format for building our query
    let fields = fields.for_sea_update();

    // -- Build query
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // -- Check result
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

    // -- Build query
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // -- Check result
    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
