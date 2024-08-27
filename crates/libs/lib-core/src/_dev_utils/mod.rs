mod dev_db;

// NOTE: OnceLock is not for async. We need OnceCell that
// supports async closure with its get_or_init()
use simple_fs::{ensure_dir, read_to_string, SFile, SPath};
use std::{collections::HashMap, fs, path::Path};
use tokio::sync::OnceCell;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::{
        self,
        task::{Task, TaskBmc, TaskForCreate},
        token::{BirdeyeRootResponse, BirdeyeTokenResponse, Token, TokenBmc, TokenForCreate},
        ModelManager,
    },
};

// TODO: Look into using serde_json on TokenForCreate struct
// (or some other Token struct) to convert TOKEN_LIST.json
// into struct. Then I could seed_tokens() and continue adding/updating
// tests.

const JAN_01_UNIX: i64 = 1704080350;
const JAN_01_HUMAN: &str = "Jan 01 2024";
const MAR_10_UNIX: i64 = 1710041950;
const MAR_10_HUMAN: &str = "Mar 10 2024";
// WARN: Troubleshooting dir locations:
// "_mock_data" ->> lib-core/_mock_data/
// "src/_dev_utils/_mock_data/" ->> lib-core/src/_dev_utils/_mock_data/
// "src/_dev_utils/_mock_data/" ->> lib-core/src/_dev_utils/_mock_data/
const MOCK_DIR: &str = "src/_dev_utils/_mock_data";
const DATA_FILE: &str = "TOKEN_LIST.json";

/// Initialize environment for local development
/// (for early development, will be called from main())
pub async fn init_dev() {
    static INIT: OnceCell<()> = OnceCell::const_new();

    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");

        // NOTE: We're breaking the rule of using unwrap(),
        // but in this case we want to fail early.
        dev_db::init_dev_db().await.unwrap();
    })
    .await;
}

/// Initialize test environment
pub async fn init_test() -> ModelManager {
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();

    let mm = INIT
        .get_or_init(|| async {
            info!("{:<12} - init_dev_test()", "FOR-DEV-TEST-ONLY");
            init_dev().await;
            ModelManager::new().await.unwrap()
        })
        .await;

    mm.clone()
}

/// Seed tasks table for testing
pub async fn seed_tasks(ctx: &Ctx, mm: &ModelManager, titles: &[&str]) -> model::Result<Vec<Task>> {
    // It's okay for our dev_utils to have a dependency on our model layer,
    // but we wouldn't want it the other way around.
    let mut tasks = Vec::new();

    for title in titles {
        let id = TaskBmc::create(
            ctx,
            mm,
            TaskForCreate {
                title: title.to_string(),
            },
        )
        .await?;

        let task = TaskBmc::get(ctx, mm, id).await?;

        tasks.push(task);
    }

    Ok(tasks)
}

/// Seed token table for testing
pub async fn seed_tokens(ctx: &Ctx, mm: &ModelManager) -> model::Result<Vec<Token>> {
    // Make sure we have a local dir, create if not
    ensure_dir(MOCK_DIR)?;

    // Q: How to read and parse JSON file?
    // REF: https://stackoverflow.com/questions/30292752/how-do-i-parse-a-json-file
    // REF: https://stackoverflow.com/questions/72289549/parsing-a-nested-json-object
    let txt = read_to_string(Path::new(MOCK_DIR).join(DATA_FILE))?;
    let root: BirdeyeRootResponse = serde_json::from_str(&txt).map_err(model::Error::SerdeJson)?;
    // Q: Can I just do 'let tokens: Vec<BirdeyeTokenResponse> = root.data.tokens;'?
    // A: Yes! Because I've already set Root { data: BirdeyeDataResponse }.
    // Q: What if I completely remove the BirdeyeDataResponse struct and just use
    // generic serde_json::Value? Dunno. This would go back to how to deser from Value.
    // My guess is to use serde_json::from_value() and then specify Vec<BirdeyeTokenResponse>
    // A: Not worth it. Keep it clear with the
    let tokens: Vec<BirdeyeTokenResponse> = root.data.tokens; // Works

    // Q: Any way to quickly seed some token details?
    // U: I used https://docs.birdeye.so/reference/get_defi-tokenlist API to fetch
    // a snapshot of all tokens and saved in _dev_utils/TOKEN_LIST.json for now.

    // Q: After adding #[serde(flatten)] timestamp: TimeStamp, how can I add shared
    // timestamp data to EACH single BirdeyeTokenResponse? If I do nothing, it errors
    // because of missing fields 'updateUnixTime' not found.
    // U: Have to pull from BirdeyeDataResponse for now. Also, need to unwrap the
    // v24h_change_percent Option<f64>, since you can't store an Option type inside PG database.
    let mut result = Vec::new();
    for token in tokens {
        let id = TokenBmc::create(
            ctx,
            mm,
            TokenForCreate {
                update_unix_time: root.data.update_unix_time,
                update_time: root.data.update_time.to_string(),
                address: token.address,
                decimals: token.decimals,
                symbol: token.symbol,
                name: token.name,
                mc: token.mc,
                v24h_change_percent: token.v24h_change_percent.unwrap_or_default(),
                v24h_usd: token.v24h_usd,
                liquidity: token.liquidity,
                logo_uri: token.logo_uri,
                last_trade_unix_time: token.last_trade_unix_time,
            },
        )
        .await?;

        let token_c = TokenBmc::get(ctx, mm, id).await?;

        result.push(token_c)
    }

    Ok(result)
}
