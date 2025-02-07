use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auditor {
    pub _id: ObjectId,
    pub user_id: ObjectId,
    pub avatar: String,
    pub first_name: String,
    pub last_name: String,
    pub about: String,
    pub company: String,
    pub free_at: String,
    pub tags: Vec<String>,
    pub contacts: Contacts,
    pub price_range: PriceRange,
    pub last_modified: i64,
    pub created_at: Option<i64>,
    pub link_id: Option<String>,
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contacts {
    pub email: Option<String>,
    pub telegram: Option<String>,
    pub public_contacts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub name: Option<String>,
    pub company: Option<String>,
    pub free_from: Option<String>,
    pub tags: Option<Vec<String>>,
    pub price_range: Option<PriceRangeFilter>,
    pub rating: Option<RangeFilter<f32>>,
    pub sort: Option<SortOption>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRangeFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeFilter<T> {
    pub from: Option<T>,
    pub to: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOption {
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "price_asc")]
    PriceAsc,
    #[serde(rename = "price_desc")]
    PriceDesc,
    #[serde(rename = "rating_asc")]
    RatingAsc,
    #[serde(rename = "rating_desc")]
    RatingDesc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub auditors: Vec<Auditor>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}