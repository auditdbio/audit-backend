use chrono::Utc;
use common::{
    access_rules::{AccessRules, Edit, Read},
    api::{seartch::delete_from_search, user::{get_by_id, new_link_id, validate_name}},
    context::GeneralContext,
    entities::{
        auditor::{ExtendedAuditor, PublicAuditor},
        contacts::Contacts,
        customer::{Customer, PublicCustomer},
        project::{Project, PublicProject},
    },
    error::{self, AddCode},
    services::{API_PREFIX, AUDITORS_SERVICE, PROTOCOL},
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
    link_id: Option<String>,
}

pub struct CustomerService {
    context: GeneralContext,
}

impl CustomerService {
    pub fn new(context: GeneralContext) -> Self {
        Self { context }
    }

    pub async fn create(&self, customer: CreateCustomer) -> error::Result<Customer<String>> {
        let auth = self.context.auth();
        let id = auth.id().ok_or(anyhow::anyhow!("No user id found"))?;

        let user = get_by_id(&self.context, auth, id.clone()).await?;

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let link_id = new_link_id(
            &self.context,
            user.name,
            id.clone(),
            true,
        ).await?;

        let customer = Customer {
            user_id: id,
            avatar: customer.avatar.unwrap_or_default(),
            first_name: customer.first_name,
            last_name: customer.last_name,
            about: customer.about.unwrap_or_default(),
            company: customer.company.unwrap_or_default(),
            contacts: customer.contacts,
            tags: customer.tags.unwrap_or_default(),
            last_modified: Utc::now().timestamp_micros(),
            created_at: Some(Utc::now().timestamp_micros()),
            link_id: Some(link_id),
        };

        customers.insert(&customer).await?;

        Ok(customer.stringify())
    }

    pub async fn find(&self, id: ObjectId) -> error::Result<Option<PublicCustomer>> {
        let auth = self.context.auth();

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let Some(customer) = customers.find("user_id", &Bson::ObjectId(id)).await? else {
            return Ok(None);
        };

        if !Read.get_access(&auth, &customer) {
            return Err(
                anyhow::anyhow!("User is not available to get data from this customer").code(403),
            );
        }

        Ok(Some(auth.public_customer(customer)))
    }

    pub async fn find_by_link_id(&self, link_id: String) -> error::Result<PublicCustomer> {
        let auth = self.context.auth();

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        if let Some(customer) = customers
            .find("link_id", &Bson::String(link_id.clone()))
            .await? {
            return Ok(auth.public_customer(customer));
        };

        let id = link_id
            .parse::<ObjectId>()
            .map_err(|_| anyhow::anyhow!("Customer not found").code(404))?;

        if let Some(customer) = self.find(id).await? {
            return Ok(customer);
        }

        Err(anyhow::anyhow!("Customer not found").code(404))
    }

    pub async fn my_customer(&self) -> error::Result<Option<Customer<String>>> {
        let auth = self.context.auth();

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let customer = customers
            .find("user_id", &Bson::ObjectId(auth.id().unwrap()))
            .await?
            .map(Customer::stringify);

        if customer.is_none() {
            let user = get_by_id(&self.context, auth, auth.id().unwrap()).await?;

            if user.current_role.to_lowercase() != "customer" {
                return Ok(None);
            }

            let has_auditor = self
                .context
                .make_request::<PublicAuditor>()
                .auth(auth)
                .get(format!(
                    "{}://{}/{}/auditor/{}",
                    PROTOCOL.as_str(),
                    AUDITORS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    auth.id().unwrap()
                ))
                .send()
                .await?
                .json::<ExtendedAuditor>()
                .await
                .is_ok();

            if has_auditor {
                return Ok(None);
            }

            let mut iter = user.name.split(' ');

            let first_name = iter.next().unwrap();
            let last_name = iter.last().unwrap_or_default();

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

    pub async fn change(&self, change: CustomerChange) -> error::Result<Customer<String>> {
        let auth = self.context.auth();
        let id = auth.id().unwrap();

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let Some(mut customer) = customers.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(&auth, &customer) {
            return Err(anyhow::anyhow!("User is not available to change this customer").code(403));
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
                let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

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

        if let Some(link_id) = change.link_id {
            if !validate_name(&link_id) {
                return Err(
                    anyhow::anyhow!("Link ID may only contain alphanumeric characters, hyphens or underscore")
                        .code(400)
                );
            }

            let new_link_id = new_link_id(
                &self.context,
                link_id,
                id,
                false,
            ).await?;

            customer.link_id = Some(new_link_id)
        }

        customer.last_modified = Utc::now().timestamp_micros();

        customers.delete("user_id", &id).await?;
        customers.insert(&customer).await?;

        Ok(customer.stringify())
    }

    pub async fn change_by_id(
        &self,
        id: ObjectId,
        change: CustomerChange
    ) -> error::Result<Customer<String>> {
        let auth = self.context.auth();

        if !auth.full_access() {
            return Err(anyhow::anyhow!("User is not available to change this customer").code(400));
        }

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;
        let Some(mut customer) = customers.find("user_id", &Bson::ObjectId(id)).await? else {
            return Err(anyhow::anyhow!("No customer found").code(400));
        };

        if change.link_id.is_some() {
            customer.link_id = change.link_id;
        }

        customer.last_modified = Utc::now().timestamp_micros();

        customers.delete("user_id", &id).await?;
        customers.insert(&customer).await?;

        Ok(customer.stringify())
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<PublicCustomer> {
        let auth = self.context.auth();

        let customers = self.context.try_get_repository::<Customer<ObjectId>>()?;

        let Some(customer) = customers.delete("user_id", &id).await? else {
            return Err(anyhow::anyhow!("No customer found").code(404));
        };

        if !Edit.get_access(&auth, &customer) {
            customers.insert(&customer).await?;
            return Err(anyhow::anyhow!("User is not available to delete this customer").code(403));
        }

        delete_from_search(&self.context, id).await?;

        Ok(auth.public_customer(customer))
    }

    pub async fn get_projects(&self, id: ObjectId) -> error::Result<Vec<PublicProject>> {
        let auth = self.context.auth();

        let projects = self.context.try_get_repository::<Project<ObjectId>>()?;

        let projects = projects
            .find_many("customer_id", &Bson::ObjectId(id))
            .await?;

        let projects = projects
            .into_iter()
            .filter(|project| Read.get_access(&auth, project))
            .map(|project| auth.public_project(project))
            .collect();

        Ok(projects)
    }
}
