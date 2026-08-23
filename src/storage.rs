use aws_sdk_s3::Client;
use aws_sdk_s3::error::DisplayErrorContext;

#[derive(Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    pub fn new(client: Client, bucket: impl Into<String>) -> Self {
        Self {
            client,
            bucket: bucket.into(),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, String> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| format!("{}", DisplayErrorContext(&error)))?;
        let bytes = object
            .body
            .collect()
            .await
            .map_err(|error| error.to_string())?
            .into_bytes()
            .to_vec();
        Ok(bytes)
    }

    pub async fn list(&self, prefix: &str) -> Result<Vec<String>, String> {
        let objects = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .map_err(|error| format!("{}", DisplayErrorContext(&error)))?;
        let mut files: Vec<_> = objects
            .contents()
            .iter()
            .filter_map(|object| object.key())
            .map(|key| key.strip_prefix(prefix).unwrap_or(key).to_owned())
            .filter(|key| {
                matches!(
                    key.rsplit('.').next(),
                    Some("jpg" | "jpeg" | "png" | "webp")
                )
            })
            .collect();
        files.sort_unstable();
        Ok(files)
    }
}
