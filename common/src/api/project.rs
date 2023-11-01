use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::project::PublicProject,
    error,
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};

pub async fn request_project(
    context: &GeneralContext,
    id: ObjectId,
    auth: Auth,
) -> error::Result<PublicProject> {
    Ok(context
        .make_request::<PublicProject>()
        .get(format!(
            "{}://{}/api/project/{}",
            PROTOCOL.as_str(),
            CUSTOMERS_SERVICE.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<PublicProject>()
        .await?)
}
