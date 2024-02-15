use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum::{body::Body, http::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use crate::crypt::token::{validate_web_token, Token};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForAuth};
use crate::model::ModelManager;
use crate::web::{set_token_cookie, Error, Result, AUTH_TOKEN};

pub async fn mw_ctx_require(
    // cookies: Cookies,
    // NOTE: !!: The BIG idea of Ctx is we're going to use it for privileges and access control,
    // at both the web layer and the model layer (access control?). So, Ctx is going to be
    // added as an argument to our web handlers and at the model layer as well.
    // NOTE: We can inject our custom Extractor in many ways (Option<Ctx>, Result<Ctx>, or directly
    // Ctx). This is a complicated topic so will need to rewatch.
    // If we just pass Ctx (instead of Result<Ctx>) and remove 'ctx?;' call,
    // it won't even run your middleware!
    // NOTE: If you require Ctx in your handlers, then this makes sure that if you
    // don't use Result<Ctx> or Option<Ctx> (i.e., you pass Ctx directly), it will error.
    // NOTE: U: Multi-crate update added a wrapper CtxW since Ctx is now external to this crate.
    // Both Ctx (lib-core) and FromRequestParts (Axum) are external to this crate.
    // REF: https://youtu.be/zUxF0kvydJs?t=597
    ctx: Result<CtxW>, // 'ctx: Ctx' - disables this mw_ctx_require()
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    // Extract the Ctx using our custom Extractor that's implemented
    // the FromRequestParts trait. Now we can use this extractor
    // in all of our routes.
    // REF: https://youtu.be/XZtlD_m59sM?t=3252
    ctx?;

    Ok(next.run(req).await)
}

// NOTE: At a high level, we don't want this fn to fail. Instead, we want
// to capture the errors and still continue processing next Middleware.
// This allows other MW or handlers to manage the error as needed.
// NOTE: U: Here's an overview of our auth resolve middleware after
// we implemented api/login password encryption and validation:
// REF: https://youtu.be/3cA_mk4vdWY?t=8732
// NOTE: U: We're going to do the heavy lifting of token validation here:
// 1. We'll use the ModelManager to access the DB to get the UserForAuth.
// 2. Tower Cookies to set the cookies
// 3. Set the Request Result (CtxExtResult), which is later retreived via
// Request Extensions from_request_parts.
pub async fn mw_ctx_resolve(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    // Again, we don't want _ctx_resolve to fail here (using '?').
    // Instead, it will be handled later downstream.
    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    // Now that we have result_ctx, we don't want to fail on this function if there
    // is an error. Instead, we need to remove the cookie if something
    // went wrong other than AuthFailNoAuthTokenCookie. If the TokenNotInCookie error,
    // then there's nothing to remove from the cookie anyway.
    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // NOTE: TIP: Nice trick. We store ctx_ext_result into a Request extension,
    // An extension of Request kinda like a data store you can insert into,
    // but by specific TYPE! You can accidentally overwrite some previous
    // value if not careful, so that's why we insert our result_ctx
    // After this, we can retrieve this result_ctx we just stored in
    // extensions by using parts.extensions.get::<Result<Ctx>>()
    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}

// NOTE: We don't want to panic if errors. Instead, we capture the entire CtxExtResult
// and then let the other MW handle specific Err cases.
async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // -- Get Token String
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    // -- Parse Token
    // Shadow 'token'variable
    // NOTE: token.parse() returns a crypt::Error, but we want a CtxExtError type.
    // We also don't capture the token info for safety reasons.
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    // -- Get UserForAuth from DB
    // REF: https://youtu.be/3cA_mk4vdWY?t=11021
    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // -- Validate Token
    validate_web_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    // -- Update Token & Cookies
    set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    // -- Create CtxExtResult to be added to Request extension
    // NOTE: Recall that CtxExtResult is independent of the web layer, so that's why
    // there is no cookie, token, etc.
    Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region: -- Ctx Extractor
// NOTE: Watch Jon Gjengset's FromRequestParts breakdown: https://youtu.be/Wnb_n5YktO8?t=2723
// NOTE: We need async-trait for our custom extractor. We use-
// Axum's FromRequestParts<State>, where S requires Send and Sync
// NOTE: There are two types of Extractors:
// -- 1. For the body (more strict)
// -- 2. For everything else (general - what we're doing it here)
// This extractor will take info from headers, URL params etc.,
// which is want we want since we're taking it from headers
// NOTE: The mw_ctx_resolve and this Ctx Extractor kinda work
// hand-in-hand. mw_ctx_resolve creates the Ctx after validating
// the auth token and then puts it (Ctx) into Request extension,
// where this Ctx Extractor will retrieve it from.
// NOTE: !! U: Multi-crate workspace update. Have to create a wrapper
// 'CtxW' for our lib-core 'Ctx', since lib-core is now external to this
// web-server crate. We will return Result<CtxW> but access inner/real
// Ctx via CtxW.0. Another alternative approach would be to implement
// FromRequestParts inside lib-core, but that defeats the whole purpose
// of the original design since lib-core would depend on Axum and the web layer!
// REF: https://youtu.be/zUxF0kvydJs?t=496
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    type Rejection = Error;

    // NOTE: Recall that our custom Result type still handles the Error (Rejection),
    // we just don't have to specify Result<Self, Self::Rejection>
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        // region: -- NEW Cookies and token components validation
        // U: After removing our previous code, we can now simply get our
        // stored Result<Ctx> type from the request extensions
        // We want to fail if we don't have our Ctx extractor.
        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)

        // endregion: -- NEW Cookies and token components validation

        // // region: -- OLD Handling cookies and our Ctx
        // // Extract user cookies using parts.extract
        // // NOTE: U: This can be expensive since our Ctx extractor is called twice
        // // on each request (for mw_ctx_require, and then for each handler)
        // // To optimize this situation, we create another mw_ctx_resolve()
        // // and moved this code there instead. Leaving here for reference.
        // let cookies = parts.extract::<Cookies>().await.unwrap();
        //
        // // Now that we have the cookies in our MW, we can do what we already
        // // did inside the mw_ctx_require fn.
        // // NOTE: Middleware has access to extractors as well, so we can use Tower's Cookies
        // // extractor to retrieve cookies from the request.
        // // This allows us to be in between the request.
        // let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
        //
        // // Parse token or return an error if authtoken not found
        // // To test this out, mess up the Cookie in the api_login()
        // let (user_id, exp, sign) = auth_token
        //     .ok_or(Error::AuthFailNoAuthTokenCookie)
        //     .and_then(parse_token)?;
        //
        // // TODO: Token components validation
        // // E.g., Typically connect to db or cache, do some hashing, etc.
        // // NOTE: This can be expensive since our Ctx extractor is called twice
        // // on each request (for mw_ctx_require, and then for each handler)
        // // To optimize this situation, we create another mw_ctx_resolve()
        // // and moved this code there instead. Leaving here for reference.

        // Ok(Ctx::new(user_id))
        // // -- endregion: -- OLD Handling cookies and our Ctx
    }
}
// endregion: -- Ctx Extractor

// region: -- Ctx Extractor Result/Error
// NOTE: This is so we don't have to make the web::Error implement
// things like Clone, etc. - this keeps the Result/Error specific
// to our Ctx Extractor.
// REF: https://youtu.be/3cA_mk4vdWY?t=10526
// NOTE: U: Adding local wrapper type CtxW for external Ctx type
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    // NOTE: Could consider having the inner model::Error instead of String
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    // NOTE: Could consider having the inner ctx::Error instead of String
    CtxCreateFail(String),
}
// endregion: -- Ctx Extractor Result/Error
