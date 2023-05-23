use anyhow::bail;
use common::{
    access_rules::{AccessRules, SendMail},
    context::Context,
    entities::letter::{CreateLetter, Letter},
    repository::Entity,
};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref FEEDBACK_EMAIL: String = std::env::var("FEEDBACK_EMAIL").unwrap();
    static ref EMAIL_ADDRESS: String = std::env::var("HELLO_MAIL_ADDRESS").unwrap();
    static ref EMAIL_PASSWORD: String = std::env::var("HELLO_MAIL_PASSWORD").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: ObjectId,
    pub name: String,
    pub company: String,
    pub email: String,
    pub message: String,
}

impl Entity for Feedback {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFeedback {
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
        MailService { context }
    }

    async fn send_email(&self, letter: Letter) -> anyhow::Result<()> {
        let email = letter.email.parse()?;

        let Ok(email) = Message::builder()
                .from(EMAIL_ADDRESS.to_string().parse().unwrap())
                .to(email)
                .subject(letter.subject.clone())
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
        Ok(())
    }

    pub async fn feedback(&self, feedback: CreateFeedback) -> anyhow::Result<()> {
        let feedbacks = self.context.try_get_repository::<Feedback>()?;

        let letter = Letter {
            id: ObjectId::new(),
            email: FEEDBACK_EMAIL.to_string(),
            message: feedback.message.clone(),
            subject: format!(
                "{} ({}) from {} send feedback",
                feedback.name, feedback.email, feedback.company
            ),
        };

        self.send_email(letter).await?;

        let feedback = Feedback {
            id: ObjectId::new(),
            name: feedback.name,
            company: feedback.company,
            email: feedback.email,
            message: feedback.message,
        };

        feedbacks.insert(&feedback).await?;
        Ok(())
    }

    pub async fn send_letter(&self, letter: CreateLetter) -> anyhow::Result<()> {
        let auth = self.context.auth();

        let letters = self.context.try_get_repository::<Letter>()?;

        if !SendMail.get_access(auth, ()) {
            bail!("Users can't send mail: {:?}", auth);
        }

        let letter = Letter {
            id: ObjectId::new(),
            subject: letter.subject,
            email: letter.email,
            message: letter.message,
        };

        letters.insert(&letter).await?;

        self.send_email(letter).await?;

        Ok(())
    }
}
