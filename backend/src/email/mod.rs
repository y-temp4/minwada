use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod mailgun;
pub mod mailhog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
}

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_email(&self, message: EmailMessage) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_email_sender() -> Box<dyn EmailSender> {
    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    match app_env.as_str() {
        "production" | "staging" => Box::new(mailgun::MailgunSender::new()),
        _ => Box::new(mailhog::MailhogSender::new()),
    }
}
