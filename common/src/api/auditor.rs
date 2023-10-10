use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::Context,
    entities::auditor::{ExtendedAuditor, PublicAuditor},
    error,
    services::{AUDITORS_SERVICE, PROTOCOL},
};

pub async fn request_auditor(
    context: &Context,
    id: ObjectId,
    auth: Auth,
) -> error::Result<ExtendedAuditor> {
    Ok(context
        .make_request::<PublicAuditor>()
        .get(format!(
            "{}://{}/api/auditor/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<ExtendedAuditor>()
        .await?)
}
