//! S3-compatible object storage behind a small trait, so tests can swap in an
//! in-memory implementation and prod can point at any S3 (RustFS, Scaleway,
//! R2, AWS) by changing env vars only.

use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;

use crate::config::Config;

const PRESIGN_TTL: Duration = Duration::from_mins(15);

#[async_trait]
pub trait Storage: Send + Sync {
    /// Presigned URL the browser PUTs the file to, bound to the content type.
    async fn presign_put(&self, key: &str, content_type: &str) -> anyhow::Result<String>;
    /// Presigned URL to download an object.
    async fn presign_get(&self, key: &str) -> anyhow::Result<String>;
    /// Whether the object exists (used to confirm an upload completed).
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct S3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Storage {
    pub fn from_config(config: &Config) -> Self {
        let credentials = Credentials::new(
            config.s3_access_key.clone(),
            config.s3_secret_key.clone(),
            None,
            None,
            "static",
        );
        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.s3_region.clone()))
            .endpoint_url(config.s3_endpoint.clone())
            .credentials_provider(credentials)
            // Bucket-in-path addressing, required by RustFS/MinIO-style endpoints.
            .force_path_style(true)
            .build();
        Self {
            client: aws_sdk_s3::Client::from_conf(s3_config),
            bucket: config.s3_bucket.clone(),
        }
    }

    fn presigning() -> anyhow::Result<PresigningConfig> {
        PresigningConfig::expires_in(PRESIGN_TTL)
            .map_err(|e| anyhow::anyhow!("invalid presigning config: {e}"))
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn presign_put(&self, key: &str, content_type: &str) -> anyhow::Result<String> {
        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(Self::presigning()?)
            .await?;
        Ok(request.uri().to_string())
    }

    async fn presign_get(&self, key: &str) -> anyhow::Result<String> {
        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(Self::presigning()?)
            .await?;
        Ok(request.uri().to_string())
    }

    async fn exists(&self, key: &str) -> anyhow::Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                if err
                    .as_service_error()
                    .is_some_and(aws_sdk_s3::operation::head_object::HeadObjectError::is_not_found)
                {
                    Ok(false)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

/// In-memory stand-in for tests: presigned URLs are fake but stable, and
/// `exists` answers from a set the test controls.
pub mod memory {
    use std::collections::HashSet;
    use std::sync::Mutex;

    use async_trait::async_trait;

    #[derive(Default)]
    pub struct MemoryStorage {
        pub objects: Mutex<HashSet<String>>,
    }

    #[async_trait]
    impl super::Storage for MemoryStorage {
        async fn presign_put(&self, key: &str, _content_type: &str) -> anyhow::Result<String> {
            // Tests treat a presign as the object appearing.
            if let Ok(mut objects) = self.objects.lock() {
                objects.insert(key.to_owned());
            }
            Ok(format!("http://storage.test/put/{key}"))
        }

        async fn presign_get(&self, key: &str) -> anyhow::Result<String> {
            Ok(format!("http://storage.test/get/{key}"))
        }

        async fn exists(&self, key: &str) -> anyhow::Result<bool> {
            Ok(self.objects.lock().is_ok_and(|o| o.contains(key)))
        }

        async fn delete(&self, key: &str) -> anyhow::Result<()> {
            if let Ok(mut objects) = self.objects.lock() {
                objects.remove(key);
            }
            Ok(())
        }
    }
}
