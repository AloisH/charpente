//! Typed configuration loaded from the environment. Missing or malformed
//! variables stop the process at boot with the offending name — never at the
//! first request that needed them.

use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    Dev,
    Prod,
}

impl AppEnv {
    pub fn is_dev(self) -> bool {
        self == Self::Dev
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app_env: AppEnv,
    #[serde(default = "default_host")]
    pub app_host: String,
    #[serde(default = "default_port")]
    pub app_port: u16,
    /// Origin of the frontend during development (Vite), used for CORS.
    #[serde(default)]
    pub app_public_url: Option<String>,
    pub database_url: String,
    /// Base64-encoded cookie signing key, at least 64 bytes once decoded.
    pub session_key: String,
    pub s3_endpoint: String,
    /// Endpoint browsers reach for presigned URLs, when it differs from
    /// `s3_endpoint` (e.g. the API talks to `http://rustfs:9000` inside the
    /// compose network while browsers hit `https://storage.example.com`).
    #[serde(default)]
    pub s3_public_endpoint: Option<String>,
    #[serde(default = "default_region")]
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    #[serde(default)]
    pub seed_admin_email: Option<String>,
    #[serde(default)]
    pub seed_admin_password: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_port() -> u16 {
    8080
}

fn default_region() -> String {
    "us-east-1".to_owned()
}

impl Config {
    /// The storage origin browsers connect to (presigned URLs, CSP).
    pub fn s3_browser_endpoint(&self) -> &str {
        self.s3_public_endpoint
            .as_deref()
            .unwrap_or(&self.s3_endpoint)
    }
}

impl Config {
    /// Load from environment variables (no prefix, names as in `.env.example`).
    pub fn from_env() -> anyhow::Result<Self> {
        let config: Self = Figment::new()
            .merge(Env::raw())
            .extract()
            .map_err(|e| anyhow::anyhow!("invalid configuration: {e}"))?;
        Ok(config)
    }
}
