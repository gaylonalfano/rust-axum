use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields, SqlBuilder, Whereable};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

// region: -- User Types
#[derive(Clone, Fields, FromRow, Serialize)]
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
    pub pwd_clear: String,
}

// NOTE: For user module impl. (e.g., inside UserBmc::create fn)
// This is when we insert a new user. Not public.
#[derive(Fields)]
struct UserForInsert {
    username: String,
}

// NOTE: Read only to validate login info.
// Used for log in logic
#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
    id: i64,
    username: String,

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
    id: i64,
    username: String,

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
        // NOTE: This fun deviates from base, so we go back to custom
        // sqlx and sqlb.
        let db = mm.db();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional::<_, E>(db)
            .await?;

        Ok(user)
    }
}

// endregion: -- UserBmc

// region: -- Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
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
        let user: UserForLogin = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context("Should have user 'demo1'")?;

        // -- Check
        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
// endregion: -- Tests
