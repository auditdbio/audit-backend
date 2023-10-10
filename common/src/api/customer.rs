use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::Context,
    entities::customer::PublicCustomer,
    error,
    services::{CUSTOMERS_SERVICE, PROTOCOL},
};

pub async fn request_customer(
    context: &Context,
    id: ObjectId,
    auth: Auth,
) -> error::Result<PublicCustomer> {
    Ok(context
        .make_request::<PublicCustomer>()
        .get(format!(
            "{}://{}/api/customer/{}",
            PROTOCOL.as_str(),
            CUSTOMERS_SERVICE.as_str(),
            id
        ))
        .auth(&auth)
        .send()
        .await?
        .json::<PublicCustomer>()
        .await?)
}
