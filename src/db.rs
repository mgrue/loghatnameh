use log::info;
use sqlx::{Pool, MySql};

pub async fn init() -> Pool<MySql> {
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    
    return sqlx::MySqlPool::connect(&db_url).await.unwrap();
}

pub async fn migrate(conn: &Pool<MySql>) {
    info!("Applying database migrations...");

    sqlx::migrate!("./migrations")
        .run(conn)
        .await
        .expect("Unable to migrate database");
}