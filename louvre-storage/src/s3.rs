use aws_sdk_s3::Client;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Other(String),
}

#[derive(Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
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
        Self { client, bucket }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
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
