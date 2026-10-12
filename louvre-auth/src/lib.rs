use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use sqlx::PgPool;
use std::{convert::Infallible, time::Duration as StdDuration};
use time::Duration;
use tokio::task::JoinHandle;
use tower_sessions::{
    Expiry, SessionManagerLayer, cookie::SameSite, session_store::ExpiredDeletion,
};
use tower_sessions_sqlx_store::PostgresStore;

#[derive(Clone)]
pub struct StudioBackend {
    account: Option<AdminAccount>,
}

#[derive(Clone)]
struct AdminAccount {
    username: String,
    password_hash: String,
}

#[derive(Clone, Debug)]
pub struct StudioUser {
    pub username: String,
    password_hash: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl StudioBackend {
    pub fn from_env() -> Self {
        match (
            std::env::var("STUDIO_USERNAME"),
            std::env::var("STUDIO_PASSWORD_HASH"),
        ) {
            (Ok(username), Ok(password_hash))
                if !username.is_empty() && !password_hash.is_empty() =>
            {
                Self::from_password_hash(username, password_hash)
            }
            _ => Self { account: None },
        }
    }

    pub fn from_password_hash(
        username: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Self {
        let username = username.into();
        let password_hash = password_hash.into();
        let account = (!username.is_empty() && !password_hash.is_empty()).then_some(AdminAccount {
            username,
            password_hash,
        });

        Self { account }
    }

    pub fn is_configured(&self) -> bool {
        self.account.is_some()
    }
}

impl AuthUser for StudioUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.username.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

impl AuthnBackend for StudioBackend {
    type User = StudioUser;
    type Credentials = Credentials;
    type Error = Infallible;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let Some(account) = &self.account else {
            return Ok(None);
        };

        let username_matches = credentials.username == account.username;
        let password = credentials.password;
        let password_hash = account.password_hash.clone();
        let hash_for_verification = password_hash.clone();
        let password_matches = tokio::task::spawn_blocking(move || {
            password_auth::verify_password(password, &hash_for_verification).is_ok()
        })
        .await
        .unwrap_or(false);

        Ok(
            (username_matches && password_matches).then_some(StudioUser {
                username: account.username.clone(),
                password_hash,
            }),
        )
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.account.as_ref().and_then(|account| {
            (account.username == *user_id).then(|| StudioUser {
                username: account.username.clone(),
                password_hash: account.password_hash.clone(),
            })
        }))
    }
}

pub fn generate_password_hash(password: impl AsRef<[u8]>) -> String {
    password_auth::generate_hash(password)
}

pub fn session_store(pool: PgPool) -> PostgresStore {
    PostgresStore::new(pool)
}

pub async fn migrate_sessions(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    session_store(pool.clone()).migrate().await?;
    Ok(())
}

pub fn session_layer(
    store: PostgresStore,
    secure_cookie: bool,
) -> SessionManagerLayer<PostgresStore> {
    SessionManagerLayer::new(store)
        .with_name("louvre_studio_session")
        .with_path("/studio")
        .with_http_only(true)
        .with_secure(secure_cookie)
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(Duration::hours(8)))
        .with_always_save(true)
}

pub fn spawn_session_cleanup(store: PostgresStore) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(error) = store
            .continuously_delete_expired(StdDuration::from_secs(60))
            .await
        {
            tracing::error!(%error, "session expiration cleanup stopped");
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn authenticates_the_configured_account() {
        let backend =
            StudioBackend::from_password_hash("owner", generate_password_hash("correct horse"));

        let user = backend
            .authenticate(Credentials {
                username: "owner".to_owned(),
                password: "correct horse".to_owned(),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id(), "owner");

        assert!(
            backend
                .authenticate(Credentials {
                    username: "owner".to_owned(),
                    password: "wrong password".to_owned(),
                })
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            backend
                .authenticate(Credentials {
                    username: "someone-else".to_owned(),
                    password: "correct horse".to_owned(),
                })
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn changed_credentials_invalidate_existing_session_auth_hashes() {
        let old_backend =
            StudioBackend::from_password_hash("owner", generate_password_hash("correct horse"));
        let old_user = old_backend
            .authenticate(Credentials {
                username: "owner".to_owned(),
                password: "correct horse".to_owned(),
            })
            .await
            .unwrap()
            .unwrap();

        let new_backend =
            StudioBackend::from_password_hash("owner", generate_password_hash("new horse"));
        let new_user = new_backend
            .get_user(&"owner".to_owned())
            .await
            .unwrap()
            .unwrap();

        assert_ne!(old_user.session_auth_hash(), new_user.session_auth_hash());
    }
}
