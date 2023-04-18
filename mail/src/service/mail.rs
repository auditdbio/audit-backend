
use anyhow::bail;
use common::{
    access_rules::{AccessRules, SendMail},
    context::Context, repository::Entity,
};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref EMAIL_ADDRESS: String = std::env::var("HELLO_MAIL_ADDRESS").unwrap();
    static ref EMAIL_PASSWORD: String = std::env::var("HELLO_MAIL_PASSWORD").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Letter {
    pub id: ObjectId,
    pub name: String,
    pub company: String,
    pub email: String,
    pub message: String,
}

impl Entity for Letter {
    fn id(&self) -> ObjectId {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLetter {
    pub name: String,
    pub company: String,
    pub email: String,
    pub message: String,

}

pub struct MailService {
    pub context: Context,
}

impl MailService {
    pub fn new(context: Context) -> MailService {
        MailService { context: context }
    }

    pub async fn send_mail(&self, letter: CreateLetter) -> anyhow::Result<()> {
        let auth = self.context.auth();

        if !SendMail::get_access(auth, ()) {
            bail!("Users can't send mail");
        }

        let letters = self.context.try_get_repository::<Letter>()?;

        let Ok(email) = letter.email.clone().parse() else {
            bail!("Error parsing email");
        };


        let Ok(email) = Message::builder()
                .from(EMAIL_ADDRESS.to_string().parse().unwrap())
                .to(email)
                .subject("Welcome to AuditDB waiting list!")
                .body(letter.message.clone()) else {
                    bail!("Error building email");
                };
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                EMAIL_ADDRESS.to_string(),
                EMAIL_PASSWORD.to_string(),
            ))
            .build();
        if let Err(err) = mailer.send(&email) {
            bail!("Error sending email: {}", err);
        }

        let letter = Letter {
            id: ObjectId::new(),
            name: letter.name,
            company: letter.company,
            email: letter.email,
            message: letter.message,
        };

        letters.insert(&letter).await?;
        Ok(())
    }
}
