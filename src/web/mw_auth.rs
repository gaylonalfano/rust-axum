use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum::{http::Request, middleware::Next, response::Response};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};

pub async fn mw_require_auth<B>(
    // cookies: Cookies,
    // NOTE: U: The BIG idea of Ctx is we're going to use it for privileges and access control,
    // at both the web layer and the model layer (access control?). So, Ctx is going to be
    // added as an argument to our web handlers and at the model layer as well.
    // NOTE: We can inject our custom Extractor in many ways (Option<Ctx>, Result<Ctx>, or directly
    // Ctx). This is a complicated topic so will need to rewatch.
    // If we just pass Ctx (instead of Result<Ctx>) and remove 'ctx?;' call,
    // it won't even run your middleware!
    // NOTE: If you require Ctx in your handlers, then this makes sure that if you
    // don't use Result<Ctx> or Option<Ctx> (i.e., you pass Ctx directly), it will error.
    ctx: Result<Ctx>, // 'ctx: Ctx' - disables this mw_require_auth()
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth", "MIDDLEWARE");

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
//
pub async fn mw_ctx_resolver<B>(
    // NOTE: Eventually you'll want to access the State ModelController,
    // which will have our database
    _mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // Compute Result<Ctx>
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, exp, sign)) => {
            // TODO: Handle the expensive Token components validation here!
            // NOTE: Apparently using match expr is more performant with async,
            // compared to closures with map(), and_then(), etc.
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };

    // Now that we have result_ctx, we don't want to fail on this function if there
    // is an error. Instead, we need to remove the cookie if something
    // went wrong other than AuthFailNoAuthTokenCookie
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::named(AUTH_TOKEN))
    }

    // NOTE: Nice trick. We store ctx_result into a Request extension,
    // An extension of Request kinda like a data store you can insert into,
    // but by specific types! You can accidentally overwrite some previous
    // value if not careful, so that's why we insert our result_ctx
    // After this, we can retrieve this result_ctx we just stored in
    // extensions by using parts.extensions.get::<Result<Ctx>>()
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// region: -- Ctx Extractor
// NOTE: We need async-trait for our custom extractor. We use-
// Axum's FromRequestParts<State>, where S requires Send and Sync
// NOTE: There are two types of Extractors:
// -- 1. For the body (more strict)
// -- 2. For everything else (general - and we're doing it here)
// This extractor will take info from headers, URL params etc.,
// which is want we want since we're taking it from headers
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    // NOTE: Recall that our custom Result type still handles the Error (Rejection),
    // we just don't have to specify Result<Self, Self::Rejection>
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        // region: -- NEW Cookies and token components validation
        // U: After removing our previous code, we can now simply get our
        // stored Result<Ctx> type from the request extensions
        // We want to fail if we don't have our Ctx extractor.
        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()

        // endregion: -- NEW Cookies and token components validation

        // // region: -- OLD Handling cookies and our Ctx
        // // Extract user cookies using parts.extract
        // // NOTE: U: This can be expensive since our Ctx extractor is called twice
        // // on each request (for mw_require_auth, and then for each handler)
        // // To optimize this situation, we create another mw_ctx_resolver()
        // // and moved this code there instead. Leaving here for reference.
        // let cookies = parts.extract::<Cookies>().await.unwrap();
        //
        // // Now that we have the cookies in our MW, we can do what we already
        // // did inside the mw_require_auth fn.
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
        // // on each request (for mw_require_auth, and then for each handler)
        // // To optimize this situation, we create another mw_ctx_resolver()
        // // and moved this code there instead. Leaving here for reference.

        // Ok(Ctx::new(user_id))
        // // -- endregion: -- OLD Handling cookies and our Ctx
    }
}
// endregion: -- Ctx Extractor

/// Parse a token of format `use-[user-id].[expiration].[signature]`
/// Returns (user_id, expiration, signature)
fn parse_token(token: String) -> Result<(u64, String, String)> {
    // Using 'lazy-regex' to compile and reuse regex expressions
    // regex_captures!() macro will parse the whole and destructure
    // the regex groups into separate variables.
    let (_whole, user_id, expiration, signature) = regex_captures!(
        r#"^user-(\d+)\.(.+)\.(.+)"#, // a literal regex
        &token
    )
    .ok_or(Error::AuthFailTokenWrongFormat)?;

    // Now we need to convert user_id &str to u64 or map to error
    // map_err() is a combinator
    let user_id: u64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, expiration.to_string(), signature.to_string()))
}
