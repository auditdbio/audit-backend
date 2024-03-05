use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::auditor::{ExtendedAuditor, PublicAuditor},
    error::{self, AddCode},
    services::{API_PREFIX, AUDITORS_SERVICE, PROTOCOL},
};

pub async fn request_auditor(
    context: &GeneralContext,
    id: ObjectId,
    auth: Auth,
) -> error::Result<ExtendedAuditor> {
    let response = context
        .make_request::<PublicAuditor>()
        .get(format!(
            "{}://{}/{}/auditor/{}",
            PROTOCOL.as_str(),
            AUDITORS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?;

    if response.status().is_success() {
        let auditor: ExtendedAuditor = response.json().await?;

        if auditor.is_empty() {
            return Err(anyhow::anyhow!("No auditor found").code(400))
        }

        Ok(auditor)
    } else {
        Err(anyhow::anyhow!("No auditor found").code(400))
    }
}
