use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    time::Duration,
};

use louvre_storage::postgres_pool;
use sqlx::{Connection, PgConnection, Row};

const USAGE: &str =
    "Usage:\n  louvre migrate\n  louvre admin hash\n  louvre admin local\n  louvre db check";

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("louvre: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    match args.as_slice() {
        ["migrate"] => migrate().await,
        ["admin", "hash"] => hash_admin_password(),
        ["admin", "local"] => configure_local_admin(),
        ["db", "check"] => check_database().await,
        ["help"] | ["--help"] | ["-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    }
}

async fn migrate() -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = connect_database().await?;
    louvre_storage::run_migrations(&database).await?;
    louvre_auth::migrate_sessions(&database).await?;
    println!("Database migrations are up to date.");
    Ok(())
}

fn hash_admin_password() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("{}", read_admin_password_hash()?);
    Ok(())
}

fn configure_local_admin() -> Result<(), Box<dyn Error + Send + Sync>> {
    print!("Studio username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    if username.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "username cannot be empty").into());
    }

    let password_hash = read_admin_password_hash()?;
    let env_path = Path::new(".env");
    let existing = match fs::read_to_string(env_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let updated = update_env_credentials(&existing, username, &password_hash);
    write_env_atomically(env_path, &updated)?;
    println!("Studio credentials saved to .env for {username}.");
    Ok(())
}

fn read_admin_password_hash() -> Result<String, Box<dyn Error + Send + Sync>> {
    let password = rpassword::prompt_password("Studio password: ")?;
    let confirmation = rpassword::prompt_password("Confirm studio password: ")?;
    if password.is_empty() || password != confirmation {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "passwords must be non-empty and match",
        )
        .into());
    }

    Ok(louvre_auth::generate_password_hash(password))
}

fn update_env_credentials(existing: &str, username: &str, password_hash: &str) -> String {
    let values = [
        ("STUDIO_USERNAME", username),
        ("STUDIO_PASSWORD_HASH", password_hash),
    ];
    let mut written = [false; 2];
    let mut updated = String::new();

    for line in existing.lines() {
        let assignment = line
            .trim_start()
            .strip_prefix("export ")
            .unwrap_or(line.trim_start());
        let key = assignment
            .split_once('=')
            .map(|(key, _)| key.trim())
            .unwrap_or_default();

        if let Some(index) = values.iter().position(|(name, _)| *name == key) {
            if !written[index] {
                updated.push_str(values[index].0);
                updated.push('=');
                updated.push_str(&quote_env_value(values[index].1));
                updated.push('\n');
                written[index] = true;
            }
        } else {
            updated.push_str(line);
            updated.push('\n');
        }
    }

    for (index, (key, value)) in values.iter().enumerate() {
        if !written[index] {
            updated.push_str(key);
            updated.push('=');
            updated.push_str(&quote_env_value(value));
            updated.push('\n');
        }
    }

    updated
}

fn quote_env_value(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

fn write_env_atomically(path: &Path, contents: &str) -> io::Result<()> {
    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let result = (|| {
        let mut file = options.open(&temp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temp_path, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp_path);
    }
    result
}

async fn check_database() -> Result<(), Box<dyn Error + Send + Sync>> {
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
                to_regclass('tower_sessions.session')::text AS sessions_table",
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
    let sessions_table: Option<String> = row.try_get("sessions_table")?;

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
    println!(
        "Session table: {}",
        if sessions_table.is_some() {
            "present"
        } else {
            "missing"
        }
    );
    if !migration_table || sessions_table.is_none() {
        eprintln!("Application schema is incomplete; run `just migrate` explicitly.");
    }

    Ok(())
}

async fn connect_database() -> Result<louvre_storage::PgPool, Box<dyn Error + Send + Sync>> {
    let database_url = database_url()?;
    Ok(postgres_pool(&database_url).await?)
}

fn database_url() -> Result<String, io::Error> {
    std::env::var("DATABASE_URL")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "DATABASE_URL must be set"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_credentials_replace_only_their_env_entries() {
        let existing = concat!(
            "# keep this comment\n",
            "DATABASE_URL=postgres://localhost/db\n",
            "STUDIO_USERNAME=old\n",
            "STUDIO_PASSWORD_HASH=old-hash\n",
            "STUDIO_USERNAME=duplicate\n",
        );

        let updated = update_env_credentials(existing, "new \"owner\"", "$argon2id$hash");

        assert!(updated.contains("# keep this comment\n"));
        assert!(updated.contains("DATABASE_URL=postgres://localhost/db\n"));
        assert!(updated.contains("STUDIO_USERNAME=\"new \\\"owner\\\"\"\n"));
        assert!(updated.contains("STUDIO_PASSWORD_HASH=\"$argon2id$hash\"\n"));
        assert_eq!(updated.matches("STUDIO_USERNAME=").count(), 1);
        assert_eq!(updated.matches("STUDIO_PASSWORD_HASH=").count(), 1);
    }
}
