use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{crypt, model::store};

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// U: Adding Serialize so log_request error can serialize into JSON
// NOTE: sqlx::Error doesn't satisfy the Serialize trait bound,
// but there's a handy serde_with::DisplayFromStr to help
#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    // NOTE: Adding a general model Error for when a get()
    // from the db doesn't return an entity (db table row item)
    EntityNotFound { entity: &'static str, id: i64 },

    // -- Modules
    // NOTE: When creating a new Model Manager, we add the Db as a
    // inner Model Controller property. However, when creating a new Db
    // using store::new_db_pool().await?; the error it could possibly
    // return is the store module's Error, NOT model module's Error.
    // Therefore, we need to expand this model Error to have a specific
    // 'store' module inner variant.
    Crypt(crypt::Error),
    Store(store::Error),

    // -- Externals
    // NOTE: sqlx::Error implements DisplayFromStr so this works
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// region: -- Froms
// Help convert these errors into the 'model' module Error
// NOTE: To allow the compiler to go from a Db Error to a Model Error,
// we have to impl From trait
impl From<crypt::Error> for Error {
    fn from(value: crypt::Error) -> Self {
        Self::Crypt(value)
    }
}

impl From<store::Error> for Error {
    fn from(value: store::Error) -> Self {
        Self::Store(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

// endregion: -- Froms

// region:  -- Error boilerplate (Optional)
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region:  -- Error boilerplate
