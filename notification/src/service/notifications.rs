pub use common::api::notifications::PublicNotification;
use common::{
    api::events::{EventPayload, PublicEvent},
    context::GeneralContext,
    entities::notification::{CreateNotification, NotificationInner},
    error::{self},
    repository::{Entity, HasLastModified},
    services::{API_PREFIX, EVENTS_SERVICE, PROTOCOL},
    default_timestamp,
    impl_has_last_modified,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::repositories::notifications::NotificationsRepository;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Notification {
    id: ObjectId,
    user_id: ObjectId,
    inner: NotificationInner,
    #[serde(default = "default_timestamp")]
    last_modified: i64,
}

impl_has_last_modified!(Notification);

impl Notification {
    pub fn serialize(self) -> PublicNotification {
        PublicNotification {
            id: self.id.to_hex(),
            user_id: self.user_id.to_hex(),
            inner: self.inner,
        }
    }
}

impl Entity for Notification {
    fn id(&self) -> ObjectId {
        self.id
    }
}

impl From<Notification> for PublicNotification {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id.to_hex(),
            user_id: notification.user_id.to_hex(),
            inner: notification.inner,
        }
    }
}

pub async fn send_notification(
    context: GeneralContext,
    notif: CreateNotification,
    notifications: &NotificationsRepository,
) -> error::Result<()> {
    let auth = context.auth();

    let notification = Notification {
        id: ObjectId::new(),
        user_id: notif.user_id,
        inner: notif.inner,
        last_modified: Utc::now().timestamp_micros(),
    };

    notifications.insert(&notification).await?;

    let event = PublicEvent::new(
        notif.user_id,
        EventPayload::Notification(notification.into()),
    );

    context
        .make_request()
        .auth(auth)
        .post(format!(
            "{}://{}/{}/event",
            PROTOCOL.as_str(),
            EVENTS_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .json(&event)
        .send()
        .await?;

    Ok(())
}

pub async fn read(
    context: GeneralContext,
    notifications: &NotificationsRepository,
    id: ObjectId,
) -> error::Result<String> {
    let _auth = context.auth();

    notifications.read(id).await?;

    Ok(id.to_hex())
}

pub async fn get_unread_notifications(
    context: GeneralContext,
    notifications: &NotificationsRepository,
) -> error::Result<Vec<PublicNotification>> {
    let auth = context.auth();

    let user_id = auth.id().unwrap();

    let notifications = notifications.get_unread(&user_id).await?;

    Ok(notifications.into_iter().map(|n| n.into()).collect())
}
