//! Outgoing mail behind a small trait, mirroring [`crate::storage`]: prod
//! points at any SMTP server via `SMTP_URL` (Mailpit in dev), tests swap in an
//! in-memory implementation, and an unset `SMTP_URL` degrades to logging the
//! mail instead of failing — the starter must run without an SMTP server.

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Config;

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()>;
}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpMailer {
    pub fn from_config(config: &Config, smtp_url: &str) -> anyhow::Result<Self> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_url)
            .map_err(|e| anyhow::anyhow!("invalid SMTP_URL: {e}"))?
            .build();
        let from = config
            .mail_from
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid MAIL_FROM: {e}"))?;
        Ok(Self { transport, from })
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
        let message = Message::builder()
            .from(self.from.clone())
            .to(to
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid recipient address: {e}"))?)
            .subject(subject)
            .body(body.to_owned())
            .map_err(|e| anyhow::anyhow!("failed to build email: {e}"))?;
        self.transport
            .send(message)
            .await
            .map_err(|e| anyhow::anyhow!("smtp send failed: {e}"))?;
        Ok(())
    }
}

/// No SMTP configured: log the mail so dev flows (and their links) stay usable.
pub struct LogMailer;

#[async_trait]
impl Mailer for LogMailer {
    async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
        tracing::info!(
            target: "wide_event",
            operation = "send_email",
            to,
            subject,
            body,
            "SMTP_URL not set — logging email instead of sending"
        );
        Ok(())
    }
}

pub fn from_config(config: &Config) -> anyhow::Result<std::sync::Arc<dyn Mailer>> {
    Ok(match config.smtp_url.as_deref() {
        Some(url) => std::sync::Arc::new(SmtpMailer::from_config(config, url)?),
        None => std::sync::Arc::new(LogMailer),
    })
}

pub mod memory {
    //! Test double capturing sent mail for assertions.

    use std::sync::Mutex;

    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    pub struct SentMail {
        pub to: String,
        pub subject: String,
        pub body: String,
    }

    #[derive(Default)]
    pub struct MemoryMailer {
        pub sent: Mutex<Vec<SentMail>>,
    }

    #[async_trait]
    impl super::Mailer for MemoryMailer {
        async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
            self.sent
                .lock()
                .map_err(|_| anyhow::anyhow!("mailer mutex poisoned"))?
                .push(SentMail {
                    to: to.to_owned(),
                    subject: subject.to_owned(),
                    body: body.to_owned(),
                });
            Ok(())
        }
    }
}
