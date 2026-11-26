mod artwork;

use std::{error::Error, io, time::Duration};

use louvre_storage::postgres_pool;
use sqlx::{Connection, PgConnection, Row};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const USAGE: &str = "Usage:\n  louvre migrate\n  louvre artwork --help\n  louvre db check";

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("louvre: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    match args.as_slice() {
        ["migrate"] => migrate().await,
        ["artwork", args @ ..] => artwork::run(args).await,
        ["db", "check"] => check_database().await,
        ["help"] | ["--help"] | ["-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    }
}

async fn migrate() -> Result<()> {
    let database = postgres_pool(&database_url()?).await?;
    louvre_storage::run_migrations(&database).await?;
    println!("Database migrations are up to date.");
    Ok(())
}

async fn check_database() -> Result<()> {
    let database_url = database_url()?;
    let mut connection =
        tokio::time::timeout(Duration::from_secs(5), PgConnection::connect(&database_url))
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "PostgreSQL connection timed out after 5 seconds (DATABASE_URL is not printed)",
                )
            })??;

    let row = sqlx::query(
        "SELECT current_database() AS database_name, \
                current_schema() AS schema_name, \
                array_to_string(current_schemas(true), ',') AS search_path, \
                host(inet_server_addr()) AS server_address, \
                inet_server_port() AS server_port, \
                to_regclass('_sqlx_migrations')::text AS migrations_table, \
                to_regclass('public._sqlx_migrations')::text AS public_migrations_table, \
                to_regclass('artworks')::text AS artworks_table",
    )
    .fetch_one(&mut connection)
    .await?;

    let database_name: String = row.try_get("database_name")?;
    let schema_name: Option<String> = row.try_get("schema_name")?;
    let search_path: Option<String> = row.try_get("search_path")?;
    let server_address: Option<String> = row.try_get("server_address")?;
    let server_port: Option<i32> = row.try_get("server_port")?;
    let migrations_table: Option<String> = row.try_get("migrations_table")?;
    let public_migrations_table: Option<String> = row.try_get("public_migrations_table")?;
    let artworks_table: Option<String> = row.try_get("artworks_table")?;

    println!("PostgreSQL connection succeeded.");
    println!("Database: {database_name}");
    println!(
        "Server: {}:{}",
        server_address.as_deref().unwrap_or("socket"),
        server_port
            .map(|port| port.to_string())
            .unwrap_or_else(|| "n/a".to_owned())
    );
    println!("Schema: {}", schema_name.as_deref().unwrap_or("none"));
    println!("Search path: {}", search_path.as_deref().unwrap_or("none"));

    let migration_table = migrations_table.is_some() || public_migrations_table.is_some();
    println!(
        "Migration ledger: {}",
        if migration_table {
            "visible"
        } else {
            "missing"
        }
    );
    if !migration_table || artworks_table.is_none() {
        eprintln!("Application schema is incomplete; run `just migrate` explicitly.");
    }

    Ok(())
}

fn database_url() -> io::Result<String> {
    std::env::var("DATABASE_URL")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "DATABASE_URL must be set"))
}
