// use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use lib_auth::pwd::{self, ContentToHash};
use modql::field::{Fields, HasFields};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

// region: -- User Types
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
    // NOTE: ! - Don't add pwd here as it will be Serialized
    // and sent back... where?
    pub id: i64,
    pub username: String,
}

// NOTE: For app api. (e.g., UserBmc::create argument)
// This UserForCreate will be sent from the Client
// or some server API, so we have Deserialize.
#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String, // Raw, unsalted, unhashed pwd (eg "welcome")
}

// NOTE: For user module impl. (e.g., inside UserBmc::create fn)
// This is when we insert a new user.
#[derive(Fields)]
pub struct UserForInsert {
    pub username: String,
}

// NOTE: Read only to validate login info.
// Used for log in logic
#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    // -- pwd and token info
    // NOTE: TIP: It's best to encode/embed your scheme
    // in your passwords from the beginning. Easier to change
    // later on.
    pub pwd: Option<String>, // encrypted, #_scheme_id_#...
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

// NOTE: Used for authentication logic.
// Kind of a subset of the UserForLogin
#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    // --token info
    pub token_salt: Uuid,
}

/// Marker trait
// NOTE: These bounds are what we have in DbBmc E (entity) type
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

// Now let's impl this UserBy trait on all of our structs,
// so now they are grouped and can work together nicely.
// It's also tidier to add all the trait bounds, lifetimes, etc. in one.
impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

// NOTE: Since the entity properties Iden will be given by modql::field::Fields, UserIden does
// not havet o be exhaustive, but just have the columns we use in our specific code.
// U: Adding Sea Query, so this enum sort of represents
// a table and its columns we are using in our custom code
// REF: https://www.youtube.com/watch?v=-dMH9UiwKqg&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q
#[derive(Iden)]
pub enum UserIden {
    Id,
    Username,
    Pwd,
}

// endregion: -- User Types

// region: -- UserBmc
pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    // NOTE: TIP: Convention is whenever calling a "first" request, then
    // we return an Option<E>, where None is acceptable return type.
    // However, we doing a "get" request, it has to be found or errors.
    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        // NOTE: This function deviates from base, so we go back to custom
        // sqlx and sqlb.
        let db = mm.db();

        // -- Build the query w/ sea-query
        // NOTE: The builder pattern in sea-query is a "Ref Mut" pattern
        // Check out my own builder-pattern repo for details!
        let mut query = Query::select();
        // NOTE:'E' is bound by 'UserBy' market trait, which impls modql 'HasFields',
        // so that's why we have E::field_idens() available.
        query
            .from(Self::table_ref())
            .columns(E::field_idens()) // similar to E::field_column_refs()
            .and_where(Expr::col(UserIden::Username).eq(username));

        // -- Exec query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let user = sqlx::query_as_with::<_, E, _>(&sql, values)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }

    pub async fn update_pwd(ctx: &Ctx, mm: &ModelManager, id: i64, pwd_clear: &str) -> Result<()> {
        let db = mm.db();

        // -- Prep password. Assumes we already have the user id
        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let pwd = pwd::hash_pwd(&ContentToHash {
            content: pwd_clear.to_string(),
            salt: user.pwd_salt,
        })?;

        // -- Build query
        let mut query = Query::update();
        query
            .table(Self::table_ref())
            .value(UserIden::Pwd, SimpleExpr::from(pwd))
            .and_where(Expr::col(UserIden::Id).eq(id));

        // -- Exec query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        // NOTE: We could consider checking this result and returning an Err or Ok
        let _count = sqlx::query_with(&sql, values)
            .execute(db)
            .await?
            .rows_affected();

        Ok(())
    }
}

// endregion: -- UserBmc

// region: -- Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>; // For early dev & tests.

    use super::*;
    use crate::_dev_utils;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        // -- Exec
        // NOTE: Cool thing is we can have user be UserForLogin or UserForAuth because
        // they ALL impl for<'r> FromRow<'r> + HasFields + Unpin + Send! Neat!
        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .ok_or("Should have user 'demo1'")?;

        // -- Check
        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
// endregion: -- Tests
