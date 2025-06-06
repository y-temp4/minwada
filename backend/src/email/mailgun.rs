use async_trait::async_trait;
use reqwest::Client;
use std::error::Error;

use super::{EmailMessage, EmailSender};

pub struct MailgunSender {
    client: Client,
}

impl MailgunSender {
    pub fn new() -> Self {
        MailgunSender {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EmailSender for MailgunSender {
    async fn send_email(&self, message: EmailMessage) -> Result<(), Box<dyn Error + Send + Sync>> {
        let api_key = std::env::var("MAILGUN_API_KEY").map_err(|_| "MAILGUN_API_KEY is not set")?;
        let domain = std::env::var("MAILGUN_DOMAIN").map_err(|_| "MAILGUN_DOMAIN is not set")?;
        let from_email =
            std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@example.com".to_string());
        let from_name =
            std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Reddit Sample".to_string());

        let from = format!("{} <{}>", from_name, from_email);
        let url = format!("https://api.mailgun.net/v3/{}/messages", domain);

        let form = reqwest::multipart::Form::new()
            .text("from", from)
            .text("to", message.to)
            .text("subject", message.subject)
            .text("html", message.html_body);

        let form = if let Some(text) = message.text_body {
            form.text("text", text)
        } else {
            form
        };

        self.client
            .post(&url)
            .basic_auth("api", Some(api_key))
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
