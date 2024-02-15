use derive_more::From;
use lib_core::model;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

// NOTE: Only our web crate errors moduls will know about Axum's
// into_response(), etc. This is for better structure instead of
// one main error. This means that previously when we added
// new modules (model, ctx, etc.) and their own errors submodule,
// we had to impl IntoResponse again and again. By making
// only this web crate to know of Axum's IntoResponse, can make
// it easier to change later on as we add more.

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// U: Adding strum_macros to have variant name as string for errors
// U: Adding Serialize so log_request error can serialize into JSON
// Handy trick when Serializing enum is to specify the tag="type" (Variant name)
// and content="data" (internal data for each variant e.g., { id: u64 })
#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- RPC
    RpcMethodUnknown(String),
    RpcMissingParams {
        rpc_method: String,
    },
    RpcFailJsonParams {
        rpc_method: String,
    },

    // -- Login
    LoginFail,
    LoginFailUsernameNotFound,
    // NOTE: TIP: Use struct variant (instead of tuple) to make
    // clear the actual value: LoginFail { user_id: i64 }.
    // Use tuple when simply holding/encapsulating the name of
    // the variant: Model(model::Error)
    LoginFailUserHasNoPwd {
        user_id: i64,
    },
    LoginFailPwdNotMatching {
        user_id: i64,
    },

    // -- Modules
    #[from]
    Model(model::Error),

    // -- External Modules
    #[from]
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:  -- Error boilerplate (Optional)
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region:  -- Error boilerplate
