use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}
