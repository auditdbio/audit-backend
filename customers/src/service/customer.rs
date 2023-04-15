use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{context::Context, entities::customer::Customer, access_rules::{Edit, AccessRules, Read}};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomer {
    avatar: String,
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    contacts: HashMap<String, String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerChange {
    avatar: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<HashMap<String, String>>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicCustomer {
    id: String,
    avatar: String,
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    contacts: HashMap<String, String>,
    tags: Vec<String>,
}


impl From<Customer<ObjectId>> for PublicCustomer {
    fn from(customer: Customer<ObjectId>) -> Self {
        Self {
            id: customer.user_id.to_hex(),
            avatar: customer.avatar,
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about,
            company: customer.company,
            contacts: customer.contacts,
            tags: customer.tags,
        }
    }
}


pub struct CustomerService {
    context: Context,
}

impl CustomerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, customer: CreateCustomer) -> anyhow::Result<PublicCustomer> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customer = Customer {
            user_id: auth.get_id().ok_or(anyhow::anyhow!("No user id found"))?.clone(),
            avatar: customer.avatar,
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about,
            company: customer.company,
            contacts: customer.contacts,
            tags: customer.tags,
            last_modified: Utc::now().timestamp_micros(),
        };

        customers.insert(&customer).await?;

        Ok(customer.into())
    }

    pub async fn find(&self, id: ObjectId) -> anyhow::Result<Option<PublicCustomer>> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(customer) = customers.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read::get_access(auth, &customer) {
            bail!("User is not available to change this customer")
        }

        Ok(Some(customer.into()))
    }

    pub async fn change(&self, id: ObjectId, change: CustomerChange) -> anyhow::Result<PublicCustomer> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(mut customer) = customers.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(auth, &customer) {
            bail!("User is not available to change this customer")
        }

        if let Some(avatar) = change.avatar {
            customer.avatar = avatar;
        }

        if let Some(first_name) = change.first_name {
            customer.first_name = first_name;
        }

        if let Some(last_name) = change.last_name {
            customer.last_name = last_name;
        }

        if let Some(about) = change.about {
            customer.about = about;
        }

        if let Some(company) = change.company {
            customer.company = company;
        }

        if let Some(contacts) = change.contacts {
            customer.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            customer.tags = tags;
        }

        customer.last_modified = Utc::now().timestamp_micros();

        customers.delete("id", &id).await?;
        customers.insert(&customer).await?;

        Ok(customer.into())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicCustomer> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(customer) = customers.find("id", &Bson::ObjectId(id)).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(auth, &customer) {
            customers.insert(&customer).await?;
            bail!("User is not available to delete this customer")
        }

        Ok(customer.into())
    }
}
