use actix_web::web;
use chrono::Utc;
use common::{context::Context, repository::RepositoryObject, auth::Auth};
use log::info;
use mongodb::bson::{Bson, Document};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::repositories::{search::SearchRepo, since::Since};

pub(super) async fn get_data(client: &Client, url: &str, since: i64) -> Option<Vec<Document>> {
    let reqwest = client.get(format!("{url}/{since}"));
    info!("Request: {:?}", reqwest);
    let Ok(res) = reqwest.send()
        .await else {
        log::error!("Error while sending request");
        return None;
    };
    info!("Response: {:?}", res);
    let Ok(body) = res.json::<Vec<Document>>().await else {
        log::error!("Error while parsing response");
        return None;
    };
    Some(body)
}

pub async fn fetch_data(auth: &Auth, since_repo: RepositoryObject<Since>, search_repo: SearchRepo) -> anyhow::Result<()> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", auth.to_token()?.parse()?);


    let client = Client::builder().default_headers(headers).build()?;
    let mut data = since_repo
        .find("name", &Bson::String("since".to_string()))
        .await
        .unwrap()
        .unwrap();

    for since in data.dict.iter_mut() {
        let timestamp = Utc::now().timestamp_micros();
        let Some(docs) = get_data(&client, &since.0 ,*since.1).await else {
            log::info!("No data for {}", since.0);
            continue;
        };
        *since.1 = timestamp;
        if docs.is_empty() {
            continue;
        }
        log::info!("Documents {:?}", docs);
        search_repo.insert(docs).await.unwrap();
    }

    since_repo.delete("id", &data.id).await.unwrap();
    since_repo.insert(&data).await.unwrap();
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub tags: String,
    pub page: u32,
    pub per_page: u32,
    pub price_from: Option<i64>,
    pub price_to: Option<i64>,
    pub time_from: Option<i64>,
    pub time_to: Option<i64>,
    pub ready_to_wait: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<i32>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInsertRequest {
    documents: Vec<Document>,
}

pub struct SearchService {
    pub repo: web::Data<SearchRepo>,
    pub context: Context,
}

impl SearchService {
    pub fn new(repo: web::Data<SearchRepo>, context: Context) -> Self {
        Self { repo, context }
    }

    pub async fn insert(&self, request: SearchInsertRequest) -> anyhow::Result<()> {
        self.repo.insert(request.documents).await?;
        Ok(())
    }

    pub async fn search(&self, query: SearchQuery) -> anyhow::Result<Vec<Document>> {
        self.repo.search(query).await
    }
}
