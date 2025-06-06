use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart},
    Message, SmtpTransport, Transport,
};
use std::error::Error;

use super::{EmailMessage, EmailSender};

pub struct MailhogSender;

impl MailhogSender {
    pub fn new() -> Self {
        MailhogSender
    }
}

#[async_trait]
impl EmailSender for MailhogSender {
    async fn send_email(&self, message: EmailMessage) -> Result<(), Box<dyn Error + Send + Sync>> {
        let from_email =
            std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@example.com".to_string());
        let from_name =
            std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Reddit Sample".to_string());

        let from = format!("{} <{}>", from_name, from_email)
            .parse::<Mailbox>()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let to = message
            .to
            .parse::<Mailbox>()
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(message.subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(
                                message
                                    .text_body
                                    .unwrap_or_else(|| message.html_body.clone()),
                            ),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(message.html_body),
                    ),
            )
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let host = std::env::var("MAILHOG_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("MAILHOG_PORT")
            .unwrap_or_else(|_| "1025".to_string())
            .parse::<u16>()
            .unwrap_or(1025);

        tracing::info!("Connecting to SMTP server at {}:{}", host, port);

        let mailer = SmtpTransport::builder_dangerous(host).port(port).build();

        // より明示的なエラーハンドリングを行う
        let send_result = tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)??;

        tracing::info!("Email sent successfully: {:?}", send_result);

        Ok(())
    }
}
