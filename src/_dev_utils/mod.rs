mod dev_db;

// NOTE: OnceLock is not for async. We need OnceCell that
// supports async closure with its get_or_init()
use tokio::sync::OnceCell;
use tracing::info;

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
