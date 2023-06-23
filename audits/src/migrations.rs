use async_trait::async_trait;
use common::entities::audit::{Audit, AuditStatus};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, Bson, Document},
    Client,
};
use mongodb_migrator::{migration::Migration, migrator::Env};

pub struct NewAuditStatusMigration {}

#[async_trait]
impl Migration for NewAuditStatusMigration {
    async fn up(&self, env: Env) -> anyhow::Result<()> {
        let conn = env
            .db
            .expect("db is unavailable")
            .collection::<Document>("audits");
        use mongodb::error::Result;
        let audits = conn
            .find(None, None)
            .await?
            .collect::<Vec<Result<Document>>>()
            .await;

        for audit in audits {
            let mut audit = audit?;
            audit.insert("status", Bson::String("Waiting".to_string()));
            let mut audit = from_document::<Audit<ObjectId>>(audit)?;
            if !audit.issues.is_empty() || audit.report.is_some() {
                audit.status = AuditStatus::Started;
            }
            conn.update_one(
                doc! {"_id": audit.id},
                doc! {"$set": {"status": serde_json::to_string(&audit.status)?}},
                None,
            )
            .await?;
        }

        Ok(())
    }
}

pub async fn up_migrations(mongo_uri: &str) -> anyhow::Result<()> {
    let client = Client::with_uri_str(mongo_uri).await.unwrap();
    let db = client.database("audits");

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(NewAuditStatusMigration {}),
        Box::new(NewAuditStatusMigration {}),
    ];
    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await?;
    Ok(())
}
