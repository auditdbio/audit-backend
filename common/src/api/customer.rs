use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    context::GeneralContext,
    entities::customer::PublicCustomer,
    error::{self, AddCode},
    services::{API_PREFIX, CUSTOMERS_SERVICE, PROTOCOL},
};

pub async fn request_customer(
    context: &GeneralContext,
    id: ObjectId,
    auth: Auth,
) -> error::Result<PublicCustomer> {
    let response = context
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
        .await?;

    if response.status().is_success() {
        let customer: PublicCustomer = response.json().await?;

        if customer.user_id.is_empty() {
            return Err(anyhow::anyhow!("No customer found").code(404))
        }

        Ok(customer)
    } else {
        Err(anyhow::anyhow!("No customer found").code(404))
    }
}
