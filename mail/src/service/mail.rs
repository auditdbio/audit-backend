use common::{
    access_rules::{AccessRules, SendMail},
    context::Context,
    entities::letter::{CreateLetter, Letter},
    repository::Entity, error::{AddCode, self},
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

    async fn send_email(&self, letter: Letter) -> error::Result<()> {
        let email = letter.email.parse()?;

        let sender_email = letter.sender.clone().unwrap_or(EMAIL_ADDRESS.to_string());

        let Ok(email) = Message::builder()
                .from(sender_email.parse().unwrap())
                .to(email)
                .subject(letter.subject.clone())
                .body(letter.message.clone()) else {
                    return Err(anyhow::anyhow!("Error building email").code(500));
                };
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                EMAIL_ADDRESS.to_string(),
                EMAIL_PASSWORD.to_string(),
            ))
            .build();
        if let Err(err) = mailer.send(&email) {
            return Err(anyhow::anyhow!("Error sending email: {}", err).code(500));
        }
        Ok(())
    }

    pub async fn feedback(&self, feedback: CreateFeedback) -> error::Result<()> {
        let feedbacks = self.context.try_get_repository::<Feedback>()?;

        let letter = Letter {
            id: ObjectId::new(),
            email: FEEDBACK_EMAIL.to_string(),
            message: feedback.message.clone(),
            subject: format!(
                "{} ({}) from {} send feedback",
                feedback.name, feedback.email, feedback.company
            ),
            sender: Some(feedback.email.clone()),
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

    pub async fn send_letter(&self, letter: CreateLetter) -> error::Result<()> {
        let auth = self.context.auth();

        let letters = self.context.try_get_repository::<Letter>()?;

        if !SendMail.get_access(auth, ()) {
            return Err(anyhow::anyhow!("Users can't send mail: {:?}", auth).code(403));
        }

        let letter = Letter {
            id: ObjectId::new(),
            subject: letter.subject,
            email: letter.email,
            message: letter.message,
            sender: Some(EMAIL_ADDRESS.to_string()),
        };

        letters.insert(&letter).await?;

        self.send_email(letter).await?;

        Ok(())
    }
}
