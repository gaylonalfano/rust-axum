use std::{fs, path::PathBuf, time::Duration};

// NOTE:
// We first execute recreate-db.sql as root_user
// Then we execute create-schema.sql and dev-seed.sql
// as the app_user.
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;

// Jeremy likes a type alias
type Db = Pool<Postgres>;

// NOTE: Hardcode to prvent deployed system db update
// POSTGRES_URL for the initial create db
// APP_URL for running all the other files
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

// sql files
const SQL_RECREATE_DB: &str = "sql/dev_initial/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

// NOTE: The Box<dyn std::error::Error> we return is not using anyhow. This is a preference.
// anyhow is used for examples and unit tests. This forces us to be
// structured from the beginning with our production application code.
pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    // -- Create the app_db/app_user with the postgres user
    // NOTE: To ensure that root_db is not accessible after its
    // intended use, we can scope its lifetime to a new block {},
    // or use the drop(root_db) function. Both work.
    {
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, SQL_RECREATE_DB).await?;
    }

    // -- Get sql files
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    // Be sure to sort the paths so we get them in order 00, 01, 02, ...
    paths.sort();

    // -- SQL execute each file
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); // for Windows

            // Only take the .sql and skip the SQL_RECREATE_DB
            // We could've added this check inside the filter_map(). Either works.
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    Ok(())
}

// Execute single sql files
async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");
    // E.g. INFO FOR-DEV-ONLY - pexec: sql/dev_initial/00-recreate-db.sql

    // -- Read the file
    let content = fs::read_to_string(file)?;

    // FIXME: Make the split for sql proof
    let sqls: Vec<&str> = content.split(";").collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}
