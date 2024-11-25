use async_trait::async_trait;
use futures::StreamExt;
use mongodb_migrator::{migration::Migration, migrator::{self, Env}};
use mongodb::{
    bson::{doc, Bson, Document, to_bson},
    Client,
};

use common::entities::{
    scope::{Scope, ScopeContent, ScopeType},
};

pub struct NewScopeProjectMigration {}

#[async_trait]
impl Migration for NewScopeProjectMigration {
    async fn up(&self, env: Env) -> anyhow::Result<()> {
        println!("Customers Migrator: Start NewScopeProjectMigration");
        let conn = env
            .db
            .expect("db is unavailable")
            .collection::<Document>("projects");
        use mongodb::error::Result;
        let projects = conn
            .find(None, None)
            .await?
            .collect::<Vec<Result<Document>>>()
            .await;

        println!("Customers Migrator: {} projects found", projects.len());

        let mut updated_documents_count = 0;
        for project in projects {
            let project = project.expect("Project document error");
            let project_id = project.get_object_id("_id").expect("get_object_id error");

            let scope = match project.get("scope") {
                Some(Bson::Array(array)) => array
                    .iter()
                    .filter_map(|bson| bson.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>(),
                _ => {
                    println!("Skipping project {:?} with invalid or missing scope field.", project_id);
                    continue
                }
            };

            let new_scope = Scope {
                typ: ScopeType::Links,
                content: ScopeContent::Links(scope),
            };

            let new_scope_bson = to_bson(&new_scope).expect("New scope to bson error")
                .as_document()
                .cloned()
                .expect("Expected BSON document");

            conn.update_one(
                doc! {"_id": project_id},
                doc! {"$set": {"scope": new_scope_bson}},
                None,
            )
                .await
                .expect("conn update_one error");
            updated_documents_count += 1;
        }

        println!("Customers Migrator: Project scope: Updated {} documents", updated_documents_count);
        Ok(())
    }
}

pub async fn up_migrations(mongo_uri: &str) -> anyhow::Result<()> {
    println!("Customers Migrator: Connecting to MongoDB...");
    println!("Customers Migrator: Using MongoDB URI: {}", mongo_uri);
    let client = Client::with_uri_str(mongo_uri)
        .await
        .expect("Customers Migrator: Error connecting to MongoDB");

    let db = client.database("customers");
    println!("Customers Migrator: Database name: {}", db.name());

    println!("Customers Migrator: Starting migrations...");
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(NewScopeProjectMigration {}),
    ];

    let result = migrator::default::DefaultMigrator::new()
        .with_conn(db.clone())
        .with_migrations_vec(migrations)
        .up()
        .await;

    match result {
        Ok(_) => println!("Customers Migrator: Migrations executed successfully."),
        Err(e) => println!("Error during migrations: {:?}", e),
    }

    Ok(())
}