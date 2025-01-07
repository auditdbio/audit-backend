use async_trait::async_trait;
use common::entities::auditor::Auditor;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Collection,
};

pub struct MongoDatabase {
    collection: Collection<Auditor<ObjectId>>,
}

impl MongoDatabase {
    pub fn new(client: Client) -> Self {
        let collection = client
            .database("auditors")
            .collection::<Auditor<ObjectId>>("auditors");
        Self { collection }
    }
}

#[async_trait]
pub trait Database {
    async fn check_presence(&self, user_ids: Vec<String>) -> anyhow::Result<Vec<bool>>;
    async fn get_modified_since(&self, timestamp: i64) -> anyhow::Result<Vec<Auditor<ObjectId>>>;
}

#[async_trait]
impl Database for MongoDatabase {
    async fn check_presence(&self, user_ids: Vec<String>) -> anyhow::Result<Vec<bool>> {
        let mut result = Vec::with_capacity(user_ids.len());
        
        for id in user_ids {
            let object_id = ObjectId::parse_str(&id)?;
            let exists = self
                .collection
                .find_one(doc! { "user_id": object_id }, None)
                .await?
                .is_some();
            result.push(exists);
        }
        
        Ok(result)
    }

    async fn get_modified_since(&self, timestamp: i64) -> anyhow::Result<Vec<Auditor<ObjectId>>> {
        let filter = doc! { "last_modified": { "$gte": timestamp } };
        let mut cursor = self.collection.find(filter, None).await?;
        
        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::entities::{
        audit_request::PriceRange,
        contacts::Contacts,
    };
    use testcontainers_modules::{
        mongo::Mongo,
        testcontainers::{runners::AsyncRunner, ContainerAsync},
    };

    fn create_sample_auditor(user_id: ObjectId, last_modified: i64) -> Auditor<ObjectId> {
        Auditor {
            user_id,
            avatar: "https://example.com/avatar.jpg".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            about: "Test auditor".to_string(),
            company: "Test Company".to_string(),
            free_at: "2024".to_string(),
            tags: vec!["rust".to_string(), "mongodb".to_string()],
            contacts: Contacts {
                email: Some("john@example.com".to_string()),
                telegram: Some("@johndoe".to_string()),
                public_contacts: true,
            },
            price_range: PriceRange {
                from: 100,
                to: 500,
            },
            last_modified,
            created_at: Some(last_modified),
            link_id: Some("test-link".to_string()),
            rating: Some(4.5),
        }
    }

    async fn get_connection_string(container: &ContainerAsync<Mongo>) -> anyhow::Result<String> {
        Ok(format!(
            "mongodb://{}:{}/",
            container.get_host().await?,
            container.get_host_port_ipv4(27017).await?,
        ))
    }

    #[tokio::test]
    async fn test_check_presence() -> anyhow::Result<()> {
        let container = Mongo::default().start().await?;
        let connection_string = get_connection_string(&container).await?;
        let client = Client::with_uri_str(&connection_string).await?;
        let db = MongoDatabase::new(client);
        
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        
        // Insert one auditor
        let auditor = create_sample_auditor(id1, 1000);
        db.collection.insert_one(auditor, None).await?;
        
        let ids = vec![id1.to_string(), id2.to_string()];
        let presence = db.check_presence(ids).await?;
        
        assert_eq!(presence, vec![true, false]);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_modified_since() -> anyhow::Result<()> {
        let container = Mongo::default().start().await?;
        let connection_string = get_connection_string(&container).await?;
        let client = Client::with_uri_str(&connection_string).await?;
        let db = MongoDatabase::new(client);
        
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        
        // Insert two auditors with different last_modified times
        let auditor1 = create_sample_auditor(id1, 1000);
        let auditor2 = create_sample_auditor(id2, 2000);
        
        db.collection.insert_one(auditor1, None).await?;
        db.collection.insert_one(auditor2, None).await?;
        
        let results = db.get_modified_since(1500).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, id2);
        Ok(())
    }
} 