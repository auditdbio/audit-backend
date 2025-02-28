use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};

use crate::{
    error::{ServiceError, ServiceResult},
    models::EntityKind,
};

use common::entities::{
    auditor::Auditor,
    badge::Badge,
    customer::Customer,
    project::Project,
};

pub struct MongoDb {
    auditors_db: Database,
    badges_db: Database,
    customers_db: Database,
}

impl MongoDb {
    pub async fn new(mongo_uri: &str) -> ServiceResult<Self> {
        let mut client_options = ClientOptions::parse(mongo_uri).await?;
        client_options.app_name = Some("fts-service".to_string());

        let client = Client::with_options(client_options)?;
        
        // Connect to different databases
        let auditors_db = client.database("auditors");
        let badges_db = client.database("badges");
        let customers_db = client.database("customers");

        Ok(Self {
            auditors_db,
            badges_db,
            customers_db,
        })
    }

    // Helper method to get the appropriate collection based on entity kind
    fn get_collection<T>(&self, kind: &EntityKind) -> Collection<T> {
        match kind {
            EntityKind::Auditor => self.auditors_db.collection("auditors"),
            EntityKind::Badge => self.badges_db.collection("badges"),
            EntityKind::Customer => self.customers_db.collection("customers"),
            EntityKind::Project => self.customers_db.collection("projects"),
        }
    }

    pub async fn get_auditors_since(&self, timestamp: i64) -> ServiceResult<Vec<Auditor<ObjectId>>> {
        let collection: Collection<Auditor<ObjectId>> = self.get_collection(&EntityKind::Auditor);
        
        let filter = doc! {
            "last_modified": { "$gt": timestamp }
        };

        let mut cursor = collection.find(filter, None).await?;
        let mut auditors = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(auditor) => auditors.push(auditor),
                Err(e) => tracing::error!("Error fetching auditor: {}", e),
            }
        }

        Ok(auditors)
    }

    pub async fn get_badges_since(&self, timestamp: i64) -> ServiceResult<Vec<Badge<ObjectId>>> {
        let collection: Collection<Badge<ObjectId>> = self.get_collection(&EntityKind::Badge);
        
        let filter = doc! {
            "last_modified": { "$gt": timestamp }
        };

        let mut cursor = collection.find(filter, None).await?;
        let mut badges = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(badge) => badges.push(badge),
                Err(e) => tracing::error!("Error fetching badge: {}", e),
            }
        }

        Ok(badges)
    }

    pub async fn get_customers_since(&self, timestamp: i64) -> ServiceResult<Vec<Customer<ObjectId>>> {
        let collection: Collection<Customer<ObjectId>> = self.get_collection(&EntityKind::Customer);
        
        let filter = doc! {
            "last_modified": { "$gt": timestamp }
        };

        let mut cursor = collection.find(filter, None).await?;
        let mut customers = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(customer) => customers.push(customer),
                Err(e) => tracing::error!("Error fetching customer: {}", e),
            }
        }

        Ok(customers)
    }

    pub async fn get_projects_since(&self, timestamp: i64) -> ServiceResult<Vec<Project<ObjectId>>> {
        let collection: Collection<Project<ObjectId>> = self.get_collection(&EntityKind::Project);
        
        let filter = doc! {
            "last_modified": { "$gt": timestamp }
        };

        let mut cursor = collection.find(filter, None).await?;
        let mut projects = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(project) => projects.push(project),
                Err(e) => tracing::error!("Error fetching project: {}", e),
            }
        }

        Ok(projects)
    }

    pub async fn get_auditors_by_ids(&self, ids: &[String]) -> ServiceResult<Vec<Auditor<ObjectId>>> {
        let collection: Collection<Auditor<ObjectId>> = self.get_collection(&EntityKind::Auditor);
        
        let object_ids: Vec<ObjectId> = ids
            .iter()
            .filter_map(|id| {
                let parsed = ObjectId::parse_str(id).map_err(|e| {
                    tracing::error!("Failed to parse ObjectId {}: {:?}", id, e);
                }).ok();
                parsed
            })
            .collect();

        let filter = doc! {
            "user_id": { "$in": object_ids.clone() }
        };

        let options = FindOptions::builder().build();

        let mut cursor = collection.find(filter, options).await?;
        let mut auditors = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(mut auditor) => {
                    if !auditor.contacts.public_contacts {
                        auditor.contacts.email = None;
                        auditor.contacts.telegram = None;
                    }
                    auditors.push(auditor)
                },
                Err(e) => tracing::error!("Error fetching auditor: {}", e),
            }
        }

        auditors.sort_by(|a, b| {
            let a_pos = ids.iter().position(|id| id == &a.user_id.to_string()).unwrap_or(usize::MAX);
            let b_pos = ids.iter().position(|id| id == &b.user_id.to_string()).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });

        Ok(auditors)
    }

    pub async fn get_badges_by_ids(&self, ids: &[String]) -> ServiceResult<Vec<Badge<ObjectId>>> {
        let collection: Collection<Badge<ObjectId>> = self.get_collection(&EntityKind::Badge);
        
        let object_ids: Vec<ObjectId> = ids
            .iter()
            .filter_map(|id| ObjectId::parse_str(id).ok())
            .collect();

        let filter = doc! {
            "user_id": { "$in": object_ids }
        };

        let options = FindOptions::builder().build();
        let mut cursor = collection.find(filter, options).await?;
        let mut badges = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(mut badge) => {
                    if !badge.contacts.public_contacts {
                        badge.contacts.email = None;
                        badge.contacts.telegram = None;
                    }
                    badges.push(badge)
                },
                Err(e) => tracing::error!("Error fetching badge: {}", e),
            }
        }

        badges.sort_by(|a, b| {
            let a_pos = ids.iter().position(|id| id == &a.user_id.to_string()).unwrap_or(usize::MAX);
            let b_pos = ids.iter().position(|id| id == &b.user_id.to_string()).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });

        Ok(badges)
    }

    pub async fn get_customers_by_ids(&self, ids: &[String]) -> ServiceResult<Vec<Customer<ObjectId>>> {
        let collection: Collection<Customer<ObjectId>> = self.get_collection(&EntityKind::Customer);
        
        let object_ids: Vec<ObjectId> = ids
            .iter()
            .filter_map(|id| ObjectId::parse_str(id).ok())
            .collect();

        let filter = doc! {
            "user_id": { "$in": object_ids }
        };

        let options = FindOptions::builder().build();
        let mut cursor = collection.find(filter, options).await?;
        let mut customers = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(mut customer) => {
                    if !customer.contacts.public_contacts {
                        customer.contacts.email = None;
                        customer.contacts.telegram = None;
                    }
                    customers.push(customer)
                },
                Err(e) => tracing::error!("Error fetching customer: {}", e),
            }
        }

        customers.sort_by(|a, b| {
            let a_pos = ids.iter().position(|id| id == &a.user_id.to_string()).unwrap_or(usize::MAX);
            let b_pos = ids.iter().position(|id| id == &b.user_id.to_string()).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });

        Ok(customers)
    }

    pub async fn get_projects_by_ids(&self, ids: &[String]) -> ServiceResult<Vec<Project<ObjectId>>> {
        let collection: Collection<Project<ObjectId>> = self.get_collection(&EntityKind::Project);
        
        let object_ids: Vec<ObjectId> = ids
            .iter()
            .filter_map(|id| ObjectId::parse_str(id).ok())
            .collect();

        let filter = doc! {
            "id": { "$in": object_ids }
        };

        let options = FindOptions::builder().build();
        let mut cursor = collection.find(filter, options).await?;
        let mut projects = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(mut project) => {
                    if !project.creator_contacts.public_contacts {
                        project.creator_contacts.email = None;
                        project.creator_contacts.telegram = None;
                    }
                    projects.push(project)
                },
                Err(e) => tracing::error!("Error fetching project: {}", e),
            }
        }

        projects.sort_by(|a, b| {
            let a_pos = ids.iter().position(|id| id == &a.id.to_string()).unwrap_or(usize::MAX);
            let b_pos = ids.iter().position(|id| id == &b.id.to_string()).unwrap_or(usize::MAX);
            a_pos.cmp(&b_pos)
        });

        Ok(projects)
    }

    pub async fn check_entity_exists(&self, kind: &EntityKind, id: &str) -> ServiceResult<bool> {
        let object_id = match ObjectId::parse_str(id) {
            Ok(oid) => oid,
            Err(e) => {
                tracing::error!("Invalid ObjectId: {}, error: {}", id, e);
                return Err(ServiceError::Query(format!("Invalid ObjectId: {}", e)));
            }
        };

        let filter = match kind {
            EntityKind::Auditor => doc! { "user_id": object_id },
            EntityKind::Badge => doc! { "user_id": object_id },
            EntityKind::Customer => doc! { "user_id": object_id },
            EntityKind::Project => doc! { "id": object_id },
        };

        let count = match kind {
            EntityKind::Auditor => {
                let collection: Collection<Auditor<ObjectId>> = self.get_collection(kind);
                collection.count_documents(filter, None).await?
            },
            EntityKind::Badge => {
                let collection: Collection<Badge<ObjectId>> = self.get_collection(kind);
                collection.count_documents(filter, None).await?
            },
            EntityKind::Customer => {
                let collection: Collection<Customer<ObjectId>> = self.get_collection(kind);
                collection.count_documents(filter, None).await?
            },
            EntityKind::Project => {
                let collection: Collection<Project<ObjectId>> = self.get_collection(kind);
                collection.count_documents(filter, None).await?
            },
        };

        Ok(count > 0)
    }

}
