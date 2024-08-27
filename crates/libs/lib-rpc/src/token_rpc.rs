use crate::params::{ParamsForCreate, ParamsForUpdate, ParamsIdOnly, ParamsList};
use crate::Result;
use lib_core::ctx::Ctx;
use lib_core::model::token::{Token, TokenBmc, TokenFilter, TokenForCreate, TokenForUpdate};
use lib_core::model::ModelManager;

// NOTE: !! - Our design is as follows: Our ModelController (TokenBmc)
// will be very granular and will only return the id (TokenBmc::create -> Result<i64>).
// But this outward facing api to return the data back (Result<Token>).
// It's these functions that directly correspond to the JSON-RPC methods.
// Eg: /api/rpc => RpcRequest => RpcRequest.method => "list_tokens" => token_rpc::list_tokens();

pub async fn create_token(
    // NOTE: This is end of line for Ctx and MM, so we're consuming
    // them both but we could pass references if we wanted.
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<TokenForCreate>,
) -> Result<Token> {
    let ParamsForCreate { data } = params;

    let id = TokenBmc::create(&ctx, &mm, data).await?;
    let token = TokenBmc::get(&ctx, &mm, id).await?;

    Ok(token)
}

pub async fn list_tokens(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsList<TokenFilter>,
) -> Result<Vec<Token>> {
    let tokens = TokenBmc::list(&ctx, &mm, params.filters, params.list_options).await?;

    Ok(tokens)
}

pub async fn update_token(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<TokenForUpdate>,
) -> Result<Token> {
    let ParamsForUpdate { id, data } = params;

    TokenBmc::update(&ctx, &mm, id, data).await?;

    let token = TokenBmc::get(&ctx, &mm, id).await?;

    Ok(token)
}

pub async fn delete_token(ctx: Ctx, mm: ModelManager, params: ParamsIdOnly) -> Result<Token> {
    let ParamsIdOnly { id } = params;

    let token = TokenBmc::get(&ctx, &mm, id).await?;
    TokenBmc::delete(&ctx, &mm, id).await?;

    Ok(token)
}
