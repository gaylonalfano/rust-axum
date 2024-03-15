mod dev_db;

// NOTE: OnceLock is not for async. We need OnceCell that
// supports async closure with its get_or_init()
use simple_fs::{ensure_dir, read_to_string};
use std::{fs, path::Path};
use tokio::sync::OnceCell;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::{
        self,
        task::{Task, TaskBmc, TaskForCreate},
        token::{Token, TokenBmc, TokenForCreate},
        ModelManager,
    },
};

// FIXME: Look into using serde_json on TokenForCreate struct
// (or some other Token struct) to convert TOKEN_LIST.json
// into struct. Then I could seed_tokens() and continue adding/updating
// tests.

const JAN_01_UNIX: i64 = 1704080350;
const MAR_10_UNIX: i64 = 1710041950;
const MOCK_DIR: &str = "_mock-data";
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

// TODO: Finish this then I can use inside model/token.rs tests module
/// Seed token table for testing
pub async fn seed_tokens(
    ctx: &Ctx,
    mm: &ModelManager,
    // symbols: &[&str],
) -> model::Result<Vec<Token>> {
    // Make sure we have a local dir
    ensure_dir(MOCK_DIR)?;

    let txt = read_to_string(Path::new(MOCK_DIR).join(DATA_FILE))?;
    println!("{}", txt);
    // let splits = simple_text_splitter(&txt, 500)?;

    let mut tokens = Vec::new();

    // Q: Any way to quickly seed some token details?
    // U: I used https://docs.birdeye.so/reference/get_defi-tokenlist API to fetch
    // a snapshot of all tokens and saved in _dev_utils/TOKEN_LIST.json for now.
    // Next, need to read that file and parse into a Token struct from json.
    //
    //
    // for symbol in symbols {
    //     let id = TokenBmc::create(ctx, mm, TokenForCreate { update_unix_time: (), update_human_time: (), address: (), decimals: (), symbol: (), name: (), mc: (), v24h_change_percent: (), v24h_usd: () })
    // }
    //
    Ok(tokens)
}
