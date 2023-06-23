use futures_util::StreamExt;
use serde::de::DeserializeOwned;

pub async fn verify<T: DeserializeOwned + Clone + Send + Sync>(
    mongo_uri: &str,
    database: &str,
    collection: &str,
    all: bool,
) -> anyhow::Result<()>
where
    mongodb::Cursor<T>:
        futures_util::StreamExt<Item = Result<T, mongodb::error::Error>> + std::marker::Unpin,
{
    let client = mongodb::Client::with_uri_str(mongo_uri).await.unwrap();
    let db = client.database(database);
    let collection = db.collection::<T>(collection);

    if all {
        let _ = collection
            .find(None, None)
            .await?
            .collect::<Vec<Result<T, mongodb::error::Error>>>()
            .await
            .into_iter()
            .collect::<Result<Vec<T>, mongodb::error::Error>>()?;
    } else {
        let value = collection.find(None, None).await?.next().await;

        if let Some(value) = value {
            let _ = value?;
        }
    }

    Ok(())
}
