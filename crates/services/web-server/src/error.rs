use derive_more::From;
use lib_core::model;

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// NOTE: As this grows, we can move into a separate 'errors' module
// U: Adding Clone so we can return our Result<Ctx, AuthFailCtxNotInRequestExt>
// from inside mw_auth.rs
// U: Adding strum_macros to have variant name as string for errors
// U: Adding Serialize so log_request error can serialize into JSON
// Handy trick when Serializing enum is to specify the tag="type" (Variant name)
// and content="data" (internal data for each variant e.g., { id: u64 })
// U: After adding "derive_more::From" dep, we don't have to manually
#[derive(Debug, From)]
pub enum Error {
    // -- Config
    ConfigMissingEnv(&'static str),
    ConfigWrongFormat(&'static str),

    // -- Modules
    #[from]
    Model(model::Error),
}

// region:  -- Froms
// U: After adding "derive_more::From" dep, we don't have to manually
// impl From<model::Error> for Error {
//     fn from(val: model::Error) -> Self {
//         Self::Model(val)
//     }
// }
// endregion:  -- Froms

// region:  -- Error boilerplate (Optional)
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region:  -- Error boilerplate
