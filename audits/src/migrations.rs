use async_trait::async_trait;
use common::entities::audit::{Audit, AuditStatus};
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, to_document, Bson, Document},
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

pub struct AuditStatusCorrection {}

#[async_trait]
impl Migration for AuditStatusCorrection {
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
                doc! {"$set": to_document(&audit)?},
                None,
            )
            .await?;
        }

        Ok(())
    }
}

pub struct SecondAttemptToMutateStatus {}

#[async_trait]
impl Migration for SecondAttemptToMutateStatus {
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

pub struct IssuesChangeWillNotFixToNotFixed {}

#[async_trait]
impl Migration for IssuesChangeWillNotFixToNotFixed {
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

            let issues = audit.get_array_mut("issues")?;
            for issue in issues {
                let issue = issue.as_document_mut().unwrap();
                if issue.get_str("status")? == "WillNotFix" {
                    issue.insert("status", "NotFixed".to_string());
                }
            }

            conn.update_one(
                doc! {"_id": audit.get_object_id("_id").unwrap()},
                doc! {"$set": audit},
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
        Box::new(SecondAttemptToMutateStatus {}),
        Box::new(AuditStatusCorrection {}),
        Box::new(IssuesChangeWillNotFixToNotFixed {}),
    ];
    mongodb_migrator::migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await?;
    Ok(())
}
