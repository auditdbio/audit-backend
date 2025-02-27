use serde::{Deserialize, Serialize};
use common::entities::auditor::Auditor;

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
    #[serde(default)]
    pub price_from: Option<String>,
    #[serde(default)]
    pub price_to: Option<String>,
    #[serde(default)]
    pub rating_from: Option<String>,
    #[serde(default)]
    pub rating_to: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_sort_option")]
    pub sort: Option<SortOption>,
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

fn deserialize_sort_option<'de, D>(deserializer: D) -> Result<Option<SortOption>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let option = Option::<String>::deserialize(deserializer)?;
    match option {
        None => Ok(Some(SortOption::Relevance)),
        Some(s) => match s.as_str().to_lowercase().as_str() {
            "relevance" => Ok(Some(SortOption::Relevance)),
            "price_asc" => Ok(Some(SortOption::PriceAsc)),
            "price_desc" => Ok(Some(SortOption::PriceDesc)),
            "rating_asc" => Ok(Some(SortOption::RatingAsc)),
            "rating_desc" => Ok(Some(SortOption::RatingDesc)),
            _ => Err(
                serde::de::Error::custom(
                    format!("Invalid sort option: {}. Acceptable values: relevance, price_asc, price_desc, rating_asc, rating_desc", s)
                )
            ),
        },
    }
}

impl SearchQuery {
    pub fn price_range(&self) -> Option<PriceRangeFilter> {
        let from = self.price_from.as_ref().and_then(|s| s.parse::<i64>().ok());
        let to = self.price_to.as_ref().and_then(|s| s.parse::<i64>().ok());
        
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
    pub result: Vec<Auditor<String>>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: usize,
    pub page: u32,
    pub per_page: u32,
}
