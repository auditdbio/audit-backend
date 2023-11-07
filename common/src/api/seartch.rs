use mongodb::bson::oid::ObjectId;

use crate::{
    context::GeneralContext,
    error,
    services::{PROTOCOL, SEARCH_SERVICE},
};

pub async fn delete_from_search(context: &GeneralContext, id: ObjectId) -> error::Result<()> {
    context
        .make_request::<()>()
        .auth(context.server_auth())
        .delete(format!(
            "{}://{}/api/search/{}",
            PROTOCOL.as_str(),
            SEARCH_SERVICE.as_str(),
            id,
        ))
        .send()
        .await?;
    Ok(())
}
