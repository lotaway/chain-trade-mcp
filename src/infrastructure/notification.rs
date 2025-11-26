use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use tracing::{error, info};
use crate::config::Config;

#[derive(Clone)]
pub struct NotificationService {
    mailer: Option<SmtpTransport>,
    from: String,
    to: String,
}

impl NotificationService {
    pub fn new(config: &Config) -> Self {
        if let (Some(host), Some(user), Some(pass), Some(from), Some(to)) = (
            &config.smtp_host,
            &config.smtp_user,
            &config.smtp_pass,
            &config.smtp_from,
            &config.smtp_to,
        ) {
            let creds = Credentials::new(user.clone(), pass.clone());
            let mailer = SmtpTransport::relay(host)
                .unwrap()
                .credentials(creds)
                .build();

            Self {
                mailer: Some(mailer),
                from: from.clone(),
                to: to.clone(),
            }
        } else {
            Self {
                mailer: None,
                from: String::new(),
                to: String::new(),
            }
        }
    }

    pub fn send_alert(&self, subject: &str, body: &str) {
        if let Some(mailer) = &self.mailer {
            let email = Message::builder()
                .from(self.from.parse().unwrap())
                .to(self.to.parse().unwrap())
                .subject(subject)
                .body(body.to_string())
                .unwrap();

            match mailer.send(&email) {
                Ok(_) => info!("Alert email sent: {}", subject),
                Err(e) => error!("Failed to send alert email: {}", e),
            }
        }
    }
}
