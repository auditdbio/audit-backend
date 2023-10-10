use common::{
    access_rules::{AccessRules, SendMail},
    context::Context,
    entities::{
        letter::{CreateLetter, Letter},
        user::PublicUser,
    },
    error::{self, AddCode},
    repository::Entity,
    services::{PROTOCOL, USERS_SERVICE},
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
        self.id
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
                .subject(letter.subject)
                .body(letter.message) else {
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

        let recipient = if let Some(recipient_name) = letter.recipient_name {
            Some(recipient_name)
        } else if let Some(recipient_id) = letter.recipient_id {
            let user = self
                .context
                .make_request::<PublicUser>()
                .auth(auth)
                .get(format!(
                    "{}://{}/api/user/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    recipient_id
                ))
                .send()
                .await?
                .json::<PublicUser>()
                .await?;
            Some(user.name)
        } else {
            None
        };

        let footer: &str = include_str!("../../templates/footer.txt");

        let message = if let Some(recipient) = recipient {
            let header =
                include_str!("../../templates/header.txt").replace("{user_name}", &recipient);
            format!("{}\n{}\n{}", header, letter.message, footer)
        } else {
            format!("{}\n{}", letter.message, footer)
        };

        let letter = Letter {
            id: ObjectId::new(),
            subject: letter.subject,
            email: letter.email,
            message,
            sender: Some(EMAIL_ADDRESS.to_string()),
        };

        letters.insert(&letter).await?;

        self.send_email(letter).await?;

        Ok(())
    }
}
