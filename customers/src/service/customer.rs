use std::collections::HashMap;

use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        customer::{Customer, PublicCustomer},
        project::Project,
    },
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomer {
    avatar: String,
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    public_contacts: bool,
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
    public_contacts: Option<bool>,
    contacts: Option<HashMap<String, String>>,
    tags: Option<Vec<String>>,
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
            user_id: auth
                .id()
                .ok_or(anyhow::anyhow!("No user id found"))?
                .clone(),
            avatar: customer.avatar,
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about,
            company: customer.company,
            contacts: customer.contacts,
            tags: customer.tags,
            last_modified: Utc::now().timestamp_micros(),
            public_contacts: customer.public_contacts,
        };

        customers.insert(&customer).await?;

        Ok(auth.public_customer(customer))
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

        Ok(Some(auth.public_customer(customer)))
    }

    pub async fn my_customer(&self) -> anyhow::Result<Option<Customer<String>>> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customer = customers
            .find("user_id", &Bson::ObjectId(auth.id().unwrap().clone()))
            .await?
            .map(Customer::stringify);

        Ok(customer)
    }

    pub async fn change(&self, change: CustomerChange) -> anyhow::Result<Customer<String>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap().clone();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(mut customer) = customers.find("user_id", &Bson::ObjectId(id)).await? else {
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

        if let Some(public_contacts) = change.public_contacts {
            customer.public_contacts = public_contacts;

            let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
                bail!("No project repository found")
            };

            let customer_projects = projects
                .find_many("customer_id", &Bson::ObjectId(id))
                .await?;

            for mut project in customer_projects {
                project.public_contacts = public_contacts;
                projects.delete("id", &project.id).await?;
                projects.insert(&project).await?;
            }
        }

        if let Some(contacts) = change.contacts {
            customer.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            customer.tags = tags;
        }

        customer.last_modified = Utc::now().timestamp_micros();

        customers.delete("user_id", &id).await?;
        customers.insert(&customer).await?;

        Ok(customer.stringify())
    }

    pub async fn delete(&self, id: ObjectId) -> anyhow::Result<PublicCustomer> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let Some(customer) = customers.delete("user_id", &id).await? else {
            bail!("No customer found")
        };

        if !Edit::get_access(auth, &customer) {
            customers.insert(&customer).await?;
            bail!("User is not available to delete this customer")
        }

        Ok(auth.public_customer(customer))
    }
}