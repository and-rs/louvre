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
struct AdminAccount {
    username: String,
    password_hash: String,
}

#[derive(Clone)]
pub struct StudioBackend {
    account: Option<AdminAccount>,
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
        let account = match (
            std::env::var("STUDIO_USERNAME"),
            std::env::var("STUDIO_PASSWORD_HASH"),
        ) {
            (Ok(username), Ok(password_hash))
                if !username.is_empty() && !password_hash.is_empty() =>
            {
                Some(AdminAccount {
                    username,
                    password_hash,
                })
            }
            _ => None,
        };

        Self { account }
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

    fn user_for(&self, username: &str) -> Option<StudioUser> {
        let account = self.account.as_ref()?;
        (account.username == username).then(|| StudioUser {
            username: account.username.clone(),
            password_hash: account.password_hash.clone(),
        })
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
        let Some(account) = self.account.as_ref() else {
            return Ok(None);
        };

        let username_matches = credentials.username == account.username;
        let password = credentials.password;
        let password_hash = account.password_hash.clone();
        let password_matches = tokio::task::spawn_blocking(move || {
            password_auth::verify_password(password, &password_hash).is_ok()
        })
        .await
        .unwrap_or(false);

        Ok(if username_matches && password_matches {
            self.user_for(&account.username)
        } else {
            None
        })
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.user_for(user_id))
    }
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

pub fn generate_password_hash(password: impl AsRef<[u8]>) -> String {
    password_auth::generate_hash(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verifies_the_configured_account_and_password() {
        let backend = StudioBackend {
            account: Some(AdminAccount {
                username: "owner".to_owned(),
                password_hash: generate_password_hash("correct horse"),
            }),
        };

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
                    password: "incorrect".to_owned(),
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
    async fn sessions_can_only_reload_the_configured_user() {
        let backend = StudioBackend {
            account: Some(AdminAccount {
                username: "owner".to_owned(),
                password_hash: generate_password_hash("correct horse"),
            }),
        };

        assert!(
            backend
                .get_user(&"owner".to_owned())
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            backend
                .get_user(&"someone-else".to_owned())
                .await
                .unwrap()
                .is_none()
        );
    }
}
