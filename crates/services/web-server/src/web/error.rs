use crate::web;
use derive_more::From;
use lib_auth::{pwd_legacy, token};
use lib_core::model;
use std::sync::Arc;

// NOTE: Only our web crate errors moduls will know about Axum's
// into_response(), etc. This is for better structure instead of
// one main error. This means that previously when we added
// new modules (model, ctx, etc.) and their own errors submodule,
// we had to impl IntoResponse again and again. By making
// only this web crate to know of Axum's IntoResponse, can make
// it easier to change later on as we add more.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use tracing::debug;

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// U: Adding strum_macros to have variant name as string for errors
// NOTE: TIP: U: Adding Serialize so log_request error can serialize into JSON
// Handy trick when Serializing enum is to specify the tag="type" (Variant name)
// and content="data" (internal data for each variant e.g., { id: u64 })
#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- Login
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

    // -- CtxExtError
    #[from]
    CtxExt(web::mw_auth::CtxExtError),

    // -- Modules
    #[from]
    Model(model::Error),
    #[from]
    Pwd(pwd_legacy::Error),
    #[from]
    Token(token::Error),
    #[from]
    Rpc(lib_rpc::Error),

    // -- External Modules
    #[from]
    SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
    // SerdeJson(String),
}

// // region:       -- Froms
// // NOTE: Added derive_more::From so no longer need manual impl blocks
// impl From<crypt::Error> for Error {
//     fn from(value: crypt::Error) -> Self {
//         Self::Crypt(value)
//     }
// }
//
// // NOTE: To allow the compiler to go from a Model Error to a Web Error,
// // we have to impl From trait
// impl From<model::Error> for Error {
//     fn from(value: model::Error) -> Self {
//         Self::Model(value)
//     }
// }
//
// // Q: How to impl Arc<serde_json::Error> ->> Error::SerdeJson(val)??
// // Do I return Self::SerdeJson(v.to_string())?
// // A: Nope... for now, not going to impl here, but instead just
// // do it mw_res_map.rs > res.extensions().get::<Arc<web::Error>>().map(Arc::as_ref);
// // U: We later add derive_more::From, so we don't have to manually impl this.
// impl From<serde_json::Error> for Error {
//     fn from(value: serde_json::Error) -> Self {
//         Self::SerdeJson(value.to_string())
//     }
// }
// // endregion:    -- Froms

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
        // NOTE: !! U: Axum 0.7 needs us to impl Clone on Error, OR we can
        // wrap Error with Arc type (see RpcInfo)
        // REF: https://youtu.be/MvWCX5ckuDE?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=283
        response.extensions_mut().insert(Arc::new(self));

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

            // -- Model
            Model(model::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id }, // Deref the &i64
            ),

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
// NOTE: U: When a client tries to interact with an entity that does not
// exist, we have a model::Error Model(EntityNotFound {..}) variant.
// We see this in our logs. But, if we want the Client to also see
// this error (in the CLIENT ERROR BODY log line), then we need to
// add a new variant here (ENTITY_NOT_FOUND) for this web::error
// module, specifically for this ClientError enum. We also need
// to add a new server-error-to-client-error mapping variant:
// (**see client_status_and_error() details)
// NOTE: U: We use serde::Serialize to Serialize the ClientError
// as JSON inside our web::mw_response_map().
// tag=VariantName, content=VariantData
// REF: https://youtu.be/3cA_mk4vdWY?t=13547
#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    SERVICE_ERROR,
}
// endregion: -- Client Error
