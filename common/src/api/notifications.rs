use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    context::GeneralContext,
    default_timestamp,
    entities::{
        letter::CreateLetter,
        notification::{CreateNotification, NotificationInner, Substitution},
        user::PublicUser,
    },
    error,
    services::{API_PREFIX, MAIL_SERVICE, NOTIFICATIONS_SERVICE, PROTOCOL, USERS_SERVICE},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNotification {
    pub id: String,
    pub user_id: String,
    pub inner: NotificationInner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewNotification {
    pub user_id: Option<ObjectId>,
    pub alert: String,
    pub subject: String,
    pub message: String,
    pub title: String,
    pub substitutions: Vec<Substitution>,
    #[serde(default)]
    pub links: Vec<String>,
    pub role: String,
}

pub async fn send_notification(
    context: &GeneralContext,
    email: bool,
    notification: bool,
    new_notification: NewNotification,
    variables: Vec<(String, String)>,
) -> error::Result<()> {
    let NewNotification {
        user_id,
        mut alert,
        mut subject,
        mut message,
        links,
        role,
        mut substitutions,
        title,
        ..
    } = new_notification;
    for (key, value) in variables {
        message = message.replace(&format!("{{{}}}", key), &value);
        subject = subject.replace(&format!("{{{}}}", key), &value);
        alert = alert.replace(&format!("{{{}}}", key), &value);
        for sub in &mut substitutions {
            sub.text = sub.text.replace(&format!("{{{}}}", key), &value);
        }
    }

    let user_id = user_id.unwrap();
    let user = context
        .make_request::<PublicUser>()
        .auth(context.server_auth())
        .get(format!(
            "{}://{}/{}/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            user_id,
        ))
        .send()
        .await?
        .json::<PublicUser>()
        .await?;
    if email {
        let create_letter = CreateLetter {
            recipient_id: user.id.parse().ok(),
            recipient_name: Some(user.name),
            email: user.email,
            message: message.clone(),
            subject: subject.clone(),
        };
        context
            .make_request::<CreateLetter>()
            .auth(context.server_auth())
            .post(format!(
                "{}://{}/{}/mail",
                PROTOCOL.as_str(),
                MAIL_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ))
            .json(&create_letter)
            .send()
            .await?;
    }

    if notification {
        let create_notification = CreateNotification {
            user_id,
            inner: NotificationInner {
                message: alert,
                substitutions,
                is_read: false,
                is_sound: true,
                links,
                timestamp: default_timestamp(),
                role,
                title: Some(title),
            },
        };
        context
            .make_request::<CreateNotification>()
            .auth(context.server_auth())
            .post(format!(
                "{}://{}/{}/send_notification",
                PROTOCOL.as_str(),
                NOTIFICATIONS_SERVICE.as_str(),
                API_PREFIX.as_str(),
            ))
            .json(&create_notification)
            .send()
            .await?;
    }

    Ok(())
}
