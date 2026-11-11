use aws_sdk_s3::Client;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Other(String),
}

#[derive(Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
    local_uploads: Option<PathBuf>,
}

impl Storage {
    pub async fn from_env() -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = if let Ok(endpoint_url) = std::env::var("S3_ENDPOINT_URL") {
            let config = aws_sdk_s3::config::Builder::from(&aws_config)
                .endpoint_url(endpoint_url)
                .force_path_style(true)
                .build();
            Client::from_conf(config)
        } else {
            Client::new(&aws_config)
        };
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "louvre-artworks".to_string());
        Self {
            client,
            bucket,
            local_uploads: None,
        }
    }

    /// Read existing S3 objects, but keep all writes and deletes on disk.
    pub fn with_local_uploads(mut self, directory: impl Into<PathBuf>) -> Self {
        self.local_uploads = Some(directory.into());
        self
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        if let Some(directory) = &self.local_uploads {
            match tokio::fs::read(local_path(directory, key)?).await {
                Ok(bytes) => return Ok(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(StorageError::Other(error.to_string())),
            }
        }
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| {
                if error
                    .as_service_error()
                    .is_some_and(|service| service.is_no_such_key())
                {
                    StorageError::NotFound
                } else {
                    StorageError::Other(format!("{}", DisplayErrorContext(&error)))
                }
            })?;
        let bytes = object
            .body
            .collect()
            .await
            .map_err(|error| StorageError::Other(error.to_string()))?
            .into_bytes()
            .to_vec();
        Ok(bytes)
    }

    pub async fn put(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError> {
        if let Some(directory) = &self.local_uploads {
            let path = local_path(directory, key)?;
            tokio::fs::create_dir_all(path.parent().expect("local object has a parent"))
                .await
                .map_err(|error| StorageError::Other(error.to_string()))?;
            return tokio::fs::write(path, bytes)
                .await
                .map_err(|error| StorageError::Other(error.to_string()));
        }
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|error| StorageError::Other(format!("{}", DisplayErrorContext(&error))))?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        if let Some(directory) = &self.local_uploads {
            return match tokio::fs::remove_file(local_path(directory, key)?).await {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(StorageError::Other(error.to_string())),
            };
        }
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| StorageError::Other(format!("{}", DisplayErrorContext(&error))))?;
        Ok(())
    }
}

fn local_path(directory: &Path, key: &str) -> Result<PathBuf, StorageError> {
    if key
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(key)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(StorageError::Other("Invalid local image key".into()));
    }
    Ok(directory.join(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_storage(directory: &Path) -> Storage {
        // No listening server: any accidental S3 write/delete makes these tests fail.
        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(aws_sdk_s3::config::Region::new("us-east-1"))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                "test", "test", None, None, "test",
            ))
            .endpoint_url("http://127.0.0.1:1")
            .force_path_style(true)
            .build();
        Storage {
            client: Client::from_conf(config),
            bucket: "production".into(),
            local_uploads: Some(directory.into()),
        }
    }

    #[tokio::test]
    async fn uploads_reads_and_deletes_stay_local() {
        let directory =
            std::env::temp_dir().join(format!("louvre-storage-{}", uuid::Uuid::new_v4()));
        let storage = local_storage(&directory);
        let key = "artworks/study/image.png";

        storage
            .put(key, b"image".to_vec(), "image/png")
            .await
            .unwrap();
        assert_eq!(storage.get(key).await.unwrap(), b"image");
        assert_eq!(
            tokio::fs::read(directory.join(key)).await.unwrap(),
            b"image"
        );
        storage.delete(key).await.unwrap();
        assert!(!directory.join(key).exists());
        storage.delete(key).await.unwrap();
        // A production-only key must never cause an S3 DELETE.
        storage
            .delete("artworks/production/image.png")
            .await
            .unwrap();
        tokio::fs::remove_dir_all(directory).await.unwrap();
    }

    #[tokio::test]
    async fn unsafe_paths_are_rejected_before_any_io() {
        let storage = local_storage(Path::new("unused"));
        for key in [
            "",
            "../outside",
            "/absolute",
            "a/../../outside",
            "a/./b",
            "a//b",
        ] {
            assert!(storage.get(key).await.is_err(), "{key}");
            assert!(
                storage.put(key, vec![], "image/png").await.is_err(),
                "{key}"
            );
            assert!(storage.delete(key).await.is_err(), "{key}");
        }
    }

    #[tokio::test]
    async fn local_read_errors_do_not_fall_back_to_s3() {
        let directory =
            std::env::temp_dir().join(format!("louvre-storage-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(directory.join("image.png"))
            .await
            .unwrap();
        let storage = local_storage(&directory);
        let error = storage.get("image.png").await.unwrap_err();
        let expected = tokio::fs::read(directory.join("image.png"))
            .await
            .unwrap_err();
        assert!(matches!(error, StorageError::Other(message) if message == expected.to_string()));
        tokio::fs::remove_dir_all(directory).await.unwrap();
    }
}
