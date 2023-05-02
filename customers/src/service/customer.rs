use anyhow::bail;
use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    context::Context,
    entities::{
        contacts::Contacts,
        customer::{Customer, PublicCustomer},
        project::Project,
        user::PublicUser,
    },
    services::{PROTOCOL, USERS_SERVICE},
};
use mongodb::bson::{oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomer {
    avatar: Option<String>,
    first_name: String,
    last_name: String,
    about: Option<String>,
    company: Option<String>,
    contacts: Contacts,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerChange {
    avatar: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<Contacts>,
    tags: Option<Vec<String>>,
}

pub struct CustomerService {
    context: Context,
}

impl CustomerService {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn create(&self, customer: CreateCustomer) -> anyhow::Result<Customer<String>> {
        let auth = self.context.auth();

        let Some(customers) = self.context.get_repository::<Customer<ObjectId>>() else {
            bail!("No customer repository found")
        };

        let customer = Customer {
            user_id: auth
                .id()
                .ok_or(anyhow::anyhow!("No user id found"))?
                .clone(),
            avatar: customer.avatar.unwrap_or_default(),
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about.unwrap_or_default(),
            company: customer.company.unwrap_or_default(),
            contacts: customer.contacts,
            tags: customer.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            is_new: true,
        };

        customers.insert(&customer).await?;

        Ok(customer.stringify())
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

        if let None = customer {
            let user = self
                .context
                .make_request::<PublicUser>()
                .auth(auth.clone())
                .get(format!(
                    "{}://{}/api/user/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    auth.id().unwrap()
                ))
                .send()
                .await?
                .json::<PublicUser>()
                .await?;

            let mut iter = user.name.split(' ');

            let first_name = iter.next().unwrap();
            let last_name = iter.next().unwrap_or_default();

            let customer = CreateCustomer {
                avatar: None,
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                about: None,
                company: None,
                contacts: Contacts {
                    email: Some(user.email),
                    telegram: None,
                    public_contacts: true,
                },
                tags: None,
            };

            let customer = self.create(customer).await?;

            return Ok(Some(customer));
        }

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

        if let Some(contacts) = change.contacts {
            if customer.contacts.public_contacts != contacts.public_contacts {
                let Some(projects) = self.context.get_repository::<Project<ObjectId>>() else {
                    bail!("No project repository found")
                };

                let customer_projects = projects
                    .find_many("customer_id", &Bson::ObjectId(id))
                    .await?;

                for mut project in customer_projects {
                    project.creator_contacts.public_contacts = contacts.public_contacts;
                    projects.delete("id", &project.id).await?;
                    projects.insert(&project).await?;
                }
            }
            customer.contacts = contacts;
        }

        if let Some(tags) = change.tags {
            customer.tags = tags;
        }

        customer.is_new = false;

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
