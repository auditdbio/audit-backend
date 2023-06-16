fn main() {
    // let client = Client::with_uri_str("mongodb://localhost:27017")
    //     .await
    //     .unwrap();
    // let db = client.database("test");
    // let users_collection = db.collection("users");

    // let migrations: Vec<Box<dyn Migration>> =
    //     vec![Box::new(M2 {}), Box::new(M3 {}), Box::new(M4 {})];
    // mongodb_migrator::migrator::default::DefaultMigrator::new()
    //     .with_conn(db.clone())
    //     .with_migrations_vec(migrations)
    //     .up()
    //     .await?;
    // let user: User<ObjectId> = users_collection.find_one(None, None).await.unwrap();
}

// pub struct MigrationExample {}

// #[async_trait]
// impl Migration for MigrationExample {
//     async fn up(&self, env: Env) -> Result<()> {
//         env.db
//             .expect("db is available")
//             .collection("test")
//             .insert_one(bson::doc! { "name": "Batman" }, None)
//             .await?;

//         println!("migration 0 up");

//         Ok(())
//     }

// }
