use mongodb::bson::oid::ObjectId;

use crate::{
    context::Context,
    entities::user::PublicUser,
    error,
    services::{PROTOCOL, USERS_SERVICE},
};

pub async fn get_by_id(context: &Context, id: ObjectId) -> error::Result<PublicUser> {
    Ok(context
        .make_request::<PublicUser>()
        .get(format!(
            "{}://{}/api/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            id
        ))
        .send()
        .await?
        .json::<PublicUser>()
        .await?)
}
