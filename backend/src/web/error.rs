// NOTE: Only our web crate errors moduls will know about Axum's
// into_response(), etc. This is for better structure instead of
// one main error. This means that previously when we added
// new modules (model, ctx, etc.) and their own errors submodule,
// we had to impl IntoResponse again and again. By making
// only this web crate to know of Axum's IntoResponse, can make
// it easier to change later on as we add more.
use crate::{crypt, model, web};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::debug;

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// U: Adding strum_macros to have variant name as string for errors
// U: Adding Serialize so log_request error can serialize into JSON
// Handy trick when Serializing enum is to specify the tag="type" (Variant name)
// and content="data" (internal data for each variant e.g., { id: u64 })
#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- RPC
    RpcMethodUnknown(String),
    RpcMissingParams { rpc_method: String },
    RpcFailJsonParams { rpc_method: String },

    // -- Login
    LoginFail,
    LoginFailUsernameNotFound,
    // NOTE: TIP: Use struct variant (instead of tuple) to make
    // clear the actual value: LoginFail { user_id: i64 }.
    // Use tuple when simply holding/encapsulating the name of
    // the variant: Model(model::Error)
    LoginFailUserHasNoPwd { user_id: i64 },
    LoginFailPwdNotMatching { user_id: i64 },

    // -- CtxExtError
    CtxExt(web::mw_auth::CtxExtError),

    // -- Modules
    Crypt(crypt::Error),
    Model(model::Error),

    // -- External Modules
    SerdeJson(String),
}

// region:       -- Froms
impl From<crypt::Error> for Error {
    fn from(value: crypt::Error) -> Self {
        Self::Crypt(value)
    }
}

// NOTE: To allow the compiler to go from a Model Error to a Web Error,
// we have to impl From trait
impl From<model::Error> for Error {
    fn from(value: model::Error) -> Self {
        Self::Model(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value.to_string())
    }
}
// endregion:    -- Froms

// region:       -- Axum IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // NOTE: NEVER pass server errors to client! For security reasons,
        // you want the lazy path being the safe path. So by default, if we
        // don't put extrawork , we don't send extra info to the client.
        debug!("{:<12} - model::Error {self:?}", "INTO_RESPONSE");

        // U: First creating a placeholder Axum response rather than returning
        // a full error response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        // Then insert our server error inside response using
        // the response.extensions_mut() store by type
        response.extensions_mut().insert(self);

        response
    }
}
// endregion:    -- Axum IntoResponse

// region:  -- Error boilerplate (Optional)
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region:  -- Error boilerplate

// region: -- Client Error
/// Convert from the root server error to the http status code and ClientError
impl Error {
    // NOTE: This allows us to customize what gets sent back to the Client whenever
    // we have certain server errors, since you don't want to send all for security.
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        // Bring our structs, enums, etc. into scope
        use web::Error::*;

        // NOTE: Optional #[allow(unreachable_patterns)] for when
        // fallback is unreachable? Not sure but it's optional. Could argue
        // you should be strict and exhaust all variants.
        #[allow(unreachable_patterns)]
        match self {
            // -- Login
            LoginFailUsernameNotFound
            | LoginFailUserHasNoPwd { .. }
            | LoginFailPwdNotMatching { .. } => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // -- Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

// After add Ctx resolver middleware, we're going to improve our
// errors for client and server to provide a bit more information
// NOTE: Client API result errors convention has all CAPS, but it's not convention
// for enums. To allow this, we need to add some macros. Also using
// strum_macros to convert variants into strings.
#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    SERVICE_ERROR,
}
// endregion: -- Client Error
