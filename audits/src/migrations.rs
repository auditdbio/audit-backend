use async_trait::async_trait;
use futures::StreamExt;
use mongodb_migrator::{migration::Migration, migrator::{self, Env}};
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, to_document, Bson, Document, to_bson},
    Client,
};

use common::entities::{
    audit::{Audit, AuditStatus},
    scope::{Scope, ScopeContent, ScopeType},
};

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

pub struct IssuesChangeNotFixedToWillNotFix {}

#[async_trait]
impl Migration for IssuesChangeNotFixedToWillNotFix {
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

        let mut updated_documents_count = 0;
        for audit in audits {
            let mut audit = audit?;

            let issues = audit.get_array_mut("issues")?;
            for issue in issues {
                let issue = issue.as_document_mut().unwrap();
                if issue.get_str("status")? == "NotFixed" {
                    updated_documents_count += 1;
                    issue.insert("status", "WillNotFix".to_string());
                }
            }

            conn.update_one(
                doc! {"_id": audit.get_object_id("_id").unwrap()},
                doc! {"$set": audit},
                None,
            )
            .await?;
        }
        println!("Updated {} documents", updated_documents_count);

        Ok(())
    }
}

pub struct NewScopeAuditMigration {}

#[async_trait]
impl Migration for NewScopeAuditMigration {
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

        let mut updated_documents_count = 0;
        for audit in audits {
            let audit = audit?;
            let audit_id = audit.get_object_id("_id")?;

            let scope = match audit.get("scope") {
                Some(Bson::Array(array)) => array
                    .iter()
                    .filter_map(|bson| bson.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>(),
                _ => continue,
            };

            let new_scope = Scope {
                typ: ScopeType::Links,
                content: ScopeContent::Links(scope),
            };

            let new_scope_bson = to_bson(&new_scope)?
                .as_document()
                .cloned()
                .expect("Expected BSON document");

            conn.update_one(
                doc! {"_id": audit_id},
                doc! {"$set": {"scope": new_scope_bson}},
                None,
            )
            .await?;
            updated_documents_count += 1;
        }

        println!("Audit scope: Updated {} documents", updated_documents_count);
        Ok(())
    }
}

pub struct NewScopeRequestMigration {}

#[async_trait]
impl Migration for NewScopeRequestMigration {
    async fn up(&self, env: Env) -> anyhow::Result<()> {
        let conn = env
            .db
            .expect("db is unavailable")
            .collection::<Document>("requests");
        use mongodb::error::Result;
        let requests = conn
            .find(None, None)
            .await?
            .collect::<Vec<Result<Document>>>()
            .await;

        let mut updated_documents_count = 0;
        for request in requests {
            let request = request?;
            let request_id = request.get_object_id("_id")?;

            let scope = match request.get("project_scope") {
                Some(Bson::Array(array)) => array
                    .iter()
                    .filter_map(|bson| bson.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>(),
                _ => continue,
            };

            let new_scope = Scope {
                typ: ScopeType::Links,
                content: ScopeContent::Links(scope),
            };

            let new_scope_bson = to_bson(&new_scope)?
                .as_document()
                .cloned()
                .expect("Expected BSON document");

            conn.update_one(
                doc! {"_id": request_id},
                doc! {"$set": {"project_scope": new_scope_bson}},
                None,
            )
                .await?;
            updated_documents_count += 1;
        }

        println!("Request scope: Updated {} documents", updated_documents_count);
        Ok(())
    }
}

pub async fn up_migrations(mongo_uri: &str) -> anyhow::Result<()> {
    let client = Client::with_uri_str(mongo_uri).await.unwrap();
    let db = client.database("audits");

    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(NewScopeRequestMigration {}),
        Box::new(NewScopeAuditMigration {}),
        Box::new(NewAuditStatusMigration {}),
        Box::new(SecondAttemptToMutateStatus {}),
        Box::new(AuditStatusCorrection {}),
        Box::new(IssuesChangeNotFixedToWillNotFix {}),
    ];
    migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await?;
    Ok(())
}
