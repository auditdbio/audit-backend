use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    default_timestamp,
    entities::{
        letter::CreateLetter,
        notification::{CreateNotification, NotificationInner},
        user::PublicUser,
    },
    error,
    services::{MAIL_SERVICE, NOTIFICATIONS_SERVICE, PROTOCOL, USERS_SERVICE},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewNotification {
    pub user_id: Option<ObjectId>,
    pub subject: String,
    pub message: String,
    #[serde(default)]
    pub links: Vec<String>,
}

pub async fn send_notification(
    context: &Context,
    email: bool,
    notification: bool,
    new_notification: NewNotification,
) -> error::Result<()> {
    let NewNotification {
        user_id,
        subject,
        message,
        links,
    } = new_notification;
    let user_id = user_id.unwrap();
    let user = context
        .make_request::<PublicUser>()
        .auth(context.server_auth())
        .get(format!(
            "{}://{}/api/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            user_id,
        ))
        .send()
        .await?
        .json::<PublicUser>()
        .await?;
    if email {
        let create_letter = CreateLetter {
            email: user.email,
            message: message.clone(),
            subject: subject.clone(),
        };
        context
            .make_request::<CreateLetter>()
            .auth(context.server_auth())
            .post(format!(
                "{}://{}/api/mail",
                PROTOCOL.as_str(),
                MAIL_SERVICE.as_str(),
            ))
            .json(&create_letter)
            .send()
            .await?;
    }

    if notification {
        let create_notification = CreateNotification {
            user_id,
            inner: NotificationInner {
                message,
                is_read: false,
                is_sound: true,
                links,
                timestamp: default_timestamp(),
            },
        };
        context
            .make_request::<CreateNotification>()
            .auth(context.server_auth())
            .post(format!(
                "{}://{}/api/mail",
                PROTOCOL.as_str(),
                NOTIFICATIONS_SERVICE.as_str(),
            ))
            .json(&create_notification)
            .send()
            .await?;
    }

    Ok(())
}
