use std::{collections::HashMap, str::FromStr};

use actix_web::web;
use chrono::Utc;
use common::{
    auth::Auth,
    context::Context,
    error::{self, AddCode},
    repository::Repository,
};

use mongodb::bson::{oid::ObjectId, Bson, Document};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::repositories::{search::SearchRepo, since::SinceRepo};

pub(super) async fn get_data(client: &Client, url: &str, since: i64) -> Option<Vec<Document>> {
    let request = client.get(format!("{url}/{since}"));
    let Ok(res) = request.send()
        .await else {
        log::error!("Error while sending request");
        return None;
    };
    let Ok(body) = res.json::<Vec<Document>>().await else {
        log::error!("Error while parsing response");
        return None;
    };

    Some(body)
}

pub async fn fetch_data(
    auth: &Auth,
    since_repo: SinceRepo,
    search_repo: SearchRepo,
) -> error::Result<()> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        ("Bearer ".to_string() + &auth.to_token()?).parse()?,
    );

    let client = Client::builder().default_headers(headers).build()?;
    let mut data = since_repo
        .find("name", &Bson::String("since".to_string()))
        .await?
        .unwrap();

    for since in data.dict.iter_mut() {
        let timestamp = Utc::now().timestamp_micros();
        let Some(docs) = get_data(&client, since.0 ,*since.1).await else {
            log::info!("No data for {}", since.0);
            continue;
        };
        *since.1 = timestamp;
        if docs.is_empty() {
            continue;
        }
        search_repo.insert(docs).await?;
    }

    since_repo.delete("id", &data.id).await.unwrap();
    since_repo.insert(&data).await.unwrap();
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub tags: String,
    pub page: u64,
    pub pages: Option<u64>,
    pub per_page: u64,
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
pub struct SearchResult {
    pub result: Vec<Document>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: u64,
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

    pub async fn insert(&self, request: SearchInsertRequest) -> error::Result<()> {
        self.repo.insert(request.documents).await?;
        Ok(())
    }

    pub async fn search(&self, query: SearchQuery) -> error::Result<SearchResult> {
        let mut auth = self.context.server_auth();
        if &Auth::None != self.context.auth() {
            auth = auth.authorized();
        }

        let SearchResult {
            total_documents,
            result,
        } = self.repo.search(query).await?;
        let mut ids: HashMap<String, Vec<ObjectId>> = HashMap::new();

        let mut indexes: HashMap<ObjectId, usize> = HashMap::new();

        for (i, doc) in result.iter().enumerate() {
            let id = ObjectId::from_str(doc.get_str("id").unwrap()).unwrap();
            let service = doc.get_str("request_url").unwrap();
            let vecs = ids.entry(service.to_string()).or_insert(Vec::new());

            indexes.insert(id, i);
            vecs.push(id);
        }

        let mut responces: Vec<Document> = Vec::new();

        for (service, ids) in ids.into_iter() {
            let docs = self
                .context
                .make_request()
                .auth(auth.clone())
                .post(service)
                .json(&ids)
                .send()
                .await?;

            let docs = docs.json::<Vec<Document>>().await?;
            responces.extend_from_slice(&docs);
        }

        let mut results = vec![Document::new(); result.len()];

        for doc in responces.iter() {
            let id = ObjectId::from_str(
                doc.get_str("id")
                    .unwrap_or_else(|_| doc.get_str("user_id").unwrap()),
            )
            .unwrap();
            let index = indexes.get(&id).unwrap();
            results[*index] = doc.clone();
        }

        log::info!("Responces: {:?}", results);

        let result = results.into_iter().filter(|doc| !doc.is_empty()).collect();

        Ok(SearchResult {
            result,
            total_documents,
        })
    }

    pub async fn delete(&self, id: ObjectId) -> error::Result<()> {
        if !matches!(self.context.auth(), Auth::Service(_, _)) {
            return Err(
                anyhow::anyhow!("You are not authorized to delete this document").code(401),
            );
        }
        self.repo.delete(id).await?;
        Ok(())
    }
}
