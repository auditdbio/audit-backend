use common::entities::customer::Customer;
use mongodb::{Collection, bson::{oid::ObjectId, doc}, Client};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct CustomerRepository {
    inner: Collection<Customer>,
}

impl CustomerRepository {
    const DATABASE: &'static str = "Customers";
    const COLLECTION: &'static str = "Customers";

    // 
    #[allow(dead_code)]
    pub async fn new(uri: String) -> Self {
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database(Self::DATABASE);
        let inner: Collection<Customer> = db.collection(Self::COLLECTION);
        Self { inner }
    }

    pub async fn create(&self, customer: Customer) -> Result<bool> {
        let exited_customer = self.find(customer.user_id).await?;

        if exited_customer.is_some() {
            return Ok(false);
        }

        self.inner.insert_one(customer, None).await?;
        Ok(true)

    }

    pub async fn find(&self, user_id: ObjectId) -> Result<Option<Customer>> {
        Ok(self.inner.find_one(doc!{"user_id": user_id}, None).await?)
    }

    pub async fn delete(&self, user_id: ObjectId) -> Result<Option<Customer>> {
        Ok(self.inner.find_one_and_delete(doc!{"user_id": user_id}, None).await?)
    }
}
