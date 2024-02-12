use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::customer::PublicCustomer,
    error,
    services::{API_PREFIX, CUSTOMERS_SERVICE, PROTOCOL},
};

pub async fn request_customer(
    context: &GeneralContext,
    id: ObjectId,
    auth: Auth,
) -> error::Result<PublicCustomer> {
    Ok(context
        .make_request::<PublicCustomer>()
        .get(format!(
            "{}://{}/{}/customer/{}",
            PROTOCOL.as_str(),
            CUSTOMERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            id
        ))
        .auth(auth)
        .send()
        .await?
        .json::<PublicCustomer>()
        .await?)
}
