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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchQuery {
    pub text: Option<String>,
    pub name: Option<String>,
    pub company: Option<String>,
    pub free_from: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_tags")]
    pub tags: Option<Vec<String>>,
    #[serde(rename = "price_range.from")]
    #[serde(default)]
    pub price_range_from: Option<String>,
    #[serde(rename = "price_range.to")]
    #[serde(default)]
    pub price_range_to: Option<String>,
    #[serde(rename = "rating.from")]
    #[serde(default)]
    pub rating_from: Option<String>,
    #[serde(rename = "rating.to")]
    #[serde(default)]
    pub rating_to: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub partial_match: Option<bool>,
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    
    match value {
        Some(s) => {
            if s.contains(',') {
                let tags: Vec<String> = s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok(Some(tags))
            } else {
                Ok(Some(vec![s]))
            }
        }
        None => Ok(None),
    }
}

impl SearchQuery {
    pub fn price_range(&self) -> Option<PriceRangeFilter> {
        let from = self.price_range_from.as_ref().and_then(|s| s.parse::<i64>().ok());
        let to = self.price_range_to.as_ref().and_then(|s| s.parse::<i64>().ok());
        
        if from.is_some() || to.is_some() {
            Some(PriceRangeFilter {
                from,
                to,
            })
        } else {
            None
        }
    }

    pub fn rating(&self) -> Option<RangeFilter<f32>> {
        let from = self.rating_from.as_ref().and_then(|s| s.parse::<f32>().ok());
        let to = self.rating_to.as_ref().and_then(|s| s.parse::<f32>().ok());
        
        if from.is_some() || to.is_some() {
            Some(RangeFilter {
                from,
                to,
            })
        } else {
            None
        }
    }
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
    pub result: Vec<Auditor>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: usize,
    pub page: u32,
    pub per_page: u32,
}