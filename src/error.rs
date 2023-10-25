use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// NOTE: Error handling best practice/normalization
// REF: https://youtu.be/XZtlD_m59sM
// CODE: https://github.com/jeremychone-channel/rust-axum-course/blob/main/src/error.rs
// Author exports this TYPE ALIAS of Result on top of this Error type.
pub type Result<T> = core::result::Result<T, Error>;

// NOTE: As this grows, we can move into a separate 'errors' module
// U: Adding Clone so we can return our Result<Ctx, AuthFailCtxNotInRequestExt>
// from inside mw_auth.rs
#[derive(Clone, Debug)]
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
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // NOTE: NEVER pass server errors to client! For security reasons,
        // you want the lazy path being the safe path. So by default, if we
        // don't put extrawork , we don't send extra info to the client.
        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()

        // let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        //
        // // Insert the Error into the response
        // response.extensions_mut().insert(self);
        //
        // response
    }
}
