use async_trait::async_trait;
use futures::StreamExt;
use mongodb_migrator::{migration::Migration, migrator::{self, Env}};
use mongodb::{
    bson::{doc, Document},
    Client,
};

pub struct AuditNotificationLinkMigration {}

#[async_trait]
impl Migration for AuditNotificationLinkMigration {
    async fn up(&self, env: Env) -> anyhow::Result<()> {
        println!("Notification Migrator: Start AuditNotificationLinkMigration");
        let conn = env
            .db
            .expect("db is unavailable")
            .collection::<Document>("notifications");
        use mongodb::error::Result;
        let notifications = conn
            .find(None, None)
            .await?
            .collect::<Vec<Result<Document>>>()
            .await;

        println!("Notification Migrator: {} notifications found", notifications.len());

        let mut updated_documents_count = 0;
        for notification in notifications {
            let notification = notification?;
            let notification_id = notification.get_object_id("_id")?;

            if let Ok(inner) = notification.get_document("inner") {
                if let Ok(links) = inner.get_array("links") {
                    let updated_links: Vec<String> = links
                        .iter()
                        .filter_map(|link| link.as_str().map(String::from))
                        .map(|link| {
                            if link.starts_with("/audit-info/") {
                                let segments: Vec<&str> = link.split('/').collect();
                                if segments.len() >= 4 && (segments[3] == "customer" || segments[3] == "auditor") {
                                    return format!("/audit/{}", segments[2]);
                                }
                            }
                            link
                        })
                        .collect();

                        conn.update_one(
                            doc! {"_id": notification_id},
                            doc! {"$set": {"inner.links": updated_links}},
                            None,
                        ).await?;
                        updated_documents_count += 1;
                }
            }
        }

        println!("Notification Migrator: Audit links updated in {} documents", updated_documents_count);
        Ok(())
    }
}

pub struct InitMigration {}
#[async_trait]
impl Migration for InitMigration {
    async fn up(&self, _env: Env) -> anyhow::Result<()> {
        Ok(())
    }
}

pub async fn up_migrations(mongo_uri: &str) -> anyhow::Result<()> {
    let client = Client::with_uri_str(mongo_uri)
        .await
        .expect("Notification Migrator: Error connecting to MongoDB");

    let db = client.database("notification");
    println!("Notification Migrator: Starting migrations...");
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(InitMigration {}),
        Box::new(AuditNotificationLinkMigration {}),
    ];

    let result = migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await;

    match result {
        Ok(_) => println!("Notification Migrator: Migrations executed successfully."),
        Err(e) => println!("Error during migrations: {:?}", e),
    }

    Ok(())
}