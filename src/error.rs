use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

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
#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFail,

    // -- Auth errors
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExt,

    // -- Model errors
    // TODO: Move to Model module
    TicketDeleteFailIdNotFound { id: u64 },
}

// region:  -- Error boilerplate (Optional)
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region:  -- Error boilerplate

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RESPONSE");

        // U: First creating a placeholder Axum response rather than returning
        // a full error response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        // Then insert our server error inside response using
        // the response.extensions_mut() store by type
        response.extensions_mut().insert(self);

        response

        // // NOTE: NEVER pass server errors to client! For security reasons,
        // // you want the lazy path being the safe path. So by default, if we
        // // don't put extrawork , we don't send extra info to the client.
        // (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()

        // let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //
        // // Insert the Error into the response
        // response.extensions_mut().insert(self);
        //
        // response
    }
}

// Convert Server Error into ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        // NOTE: Optional #[allow(unreachable_patterns)] for when
        // fallback is unreachable? Not sure but it's optional. Could argue
        // you should be strict and exhaust all variants.
        #[allow(unreachable_patterns)]
        match self {
            // - Login
            // TODO: Revise our Server side LoginFail Error and DON'T send
            // back to client side for security
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // - Auth
            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // - Model
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // - Fallback
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
    INVALID_PARAMS,
    SERVICE_ERROR,
}
