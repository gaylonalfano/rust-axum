mod dev_db;

// NOTE: OnceLock is not for async. We need OnceCell that
// supports async closure with its get_or_init()
use tokio::sync::OnceCell;
use tracing::info;

use crate::{
    ctx::Ctx,
    model::{
        self,
        task::{Task, TaskBmc, TaskForCreate},
        ModelManager,
    },
};

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
