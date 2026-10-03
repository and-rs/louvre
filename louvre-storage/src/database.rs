use diesel::{QueryableByName, sql_query, sql_types::Text};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub type DbPool = diesel_async::pooled_connection::bb8::Pool<AsyncPgConnection>;

pub async fn postgres_pool(database_url: &str) -> DbPool {
    let manager =
        diesel_async::pooled_connection::AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            database_url,
        );
    let pool = diesel_async::pooled_connection::bb8::Pool::builder()
        .build(manager)
        .await
        .expect("failed to initialize PostgreSQL connection pool");
    let mut connection = pool.get().await.expect("failed to connect to PostgreSQL");

    sql_query(
        "CREATE TABLE IF NOT EXISTS hello_message (
            id SMALLINT PRIMARY KEY CHECK (id = 1),
            message TEXT NOT NULL
        )",
    )
    .execute(&mut connection)
    .await
    .expect("failed to create hello_message table");
    sql_query(
        "INSERT INTO hello_message (id, message) VALUES (1, 'Hello, world!')
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(&mut connection)
    .await
    .expect("failed to seed hello_message table");
    drop(connection);

    pool
}

#[derive(QueryableByName)]
struct HelloMessage {
    #[diesel(sql_type = Text)]
    message: String,
}

pub async fn hello(connection: &mut AsyncPgConnection) -> Result<String, diesel::result::Error> {
    let result = sql_query("SELECT message FROM hello_message WHERE id = 1")
        .get_result::<HelloMessage>(connection)
        .await?;
    Ok(result.message)
}
