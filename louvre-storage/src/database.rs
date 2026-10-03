use diesel::{QueryableByName, sql_query, sql_types::Text};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub type DbPool = diesel_async::pooled_connection::bb8::Pool<AsyncPgConnection>;

pub async fn postgres_pool(database_url: &str) -> DbPool {
    let manager =
        diesel_async::pooled_connection::AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            database_url,
        );
    diesel_async::pooled_connection::bb8::Pool::builder()
        .build(manager)
        .await
        .expect("failed to initialize PostgreSQL connection pool")
}

#[derive(QueryableByName)]
struct HelloMessage {
    #[diesel(sql_type = Text)]
    message: String,
}

pub async fn hello(connection: &mut AsyncPgConnection) -> Result<String, diesel::result::Error> {
    let result = sql_query("SELECT 'Hello, world!' AS message")
        .get_result::<HelloMessage>(connection)
        .await?;
    Ok(result.message)
}
