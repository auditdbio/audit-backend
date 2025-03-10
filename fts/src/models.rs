use serde::{Deserialize, Serialize};
use common::entities::{
    auditor::Auditor,
    badge::Badge,
    customer::Customer,
    project::Project,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchQuery {
    pub q: Option<String>,
    pub text: Option<String>,
    pub name: Option<String>,
    pub company: Option<String>,
    pub free_at: Option<String>,
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
    #[serde(deserialize_with = "deserialize_kind")]
    pub kind: Vec<EntityKind>,
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
    pub fn parse_q(&mut self) {
        if let Some(q_param) = &self.q {
            let parts = split_query(q_param);
            
            let mut text_parts = Vec::new();
            
            for part in parts {
                if part.contains(':') {
                    let mut key_value = part.splitn(2, ':');
                    if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                        match key {
                            "name" => {
                                if self.name.is_none() {
                                    self.name = Some(value.to_string());
                                }
                            },
                            "company" => {
                                if self.company.is_none() {
                                    self.company = Some(value.to_string());
                                }
                            },
                            "tags" => {
                                if self.tags.is_none() {
                                    self.tags = Some(value.split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect());
                                }
                            },
                            "free_at" => {
                                if self.free_at.is_none() {
                                    self.free_at = Some(value.to_string());
                                }
                            },
                            "price" => {
                                let (from, to) = parse_range_value(value);
                                if from.is_some() && self.price_from.is_none() {
                                    self.price_from = from;
                                }
                                if to.is_some() && self.price_to.is_none() {
                                    self.price_to = to;
                                }
                            },
                            "rating" => {
                                let (from, to) = parse_range_value(value);
                                if from.is_some() && self.rating_from.is_none() {
                                    self.rating_from = from;
                                }
                                if to.is_some() && self.rating_to.is_none() {
                                    self.rating_to = to;
                                }
                            },
                            _ => {
                                text_parts.push(part.to_string());
                            }
                        }
                    }
                } else {
                    text_parts.push(part.to_string());
                }
            }
            
            if !text_parts.is_empty() && self.text.is_none() {
                self.text = Some(text_parts.join(" "));
            }
        }
    }
    
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
    pub result: Vec<SearchResult>,
    #[serde(rename = "totalDocuments")]
    pub total_documents: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SearchResult {
    #[serde(rename = "auditor")]
    Auditor(Auditor<String>),
    #[serde(rename = "badge")]
    Badge(Badge<String>),
    #[serde(rename = "customer")]
    Customer(Customer<String>),
    #[serde(rename = "project")]
    Project(Project<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityKind {
    Auditor,
    Badge,
    Customer,
    Project,
}

impl Default for EntityKind {
    fn default() -> Self {
        EntityKind::Auditor
    }
}

fn deserialize_kind<'de, D>(deserializer: D) -> Result<Vec<EntityKind>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    
    match value {
        Some(s) => {
            if s.contains(',') {
                let kinds: Vec<EntityKind> = s.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .map(|s| match s.as_str() {
                        "auditor" => Ok(EntityKind::Auditor),
                        "badge" => Ok(EntityKind::Badge),
                        "customer" => Ok(EntityKind::Customer),
                        "project" => Ok(EntityKind::Project),
                        _ => Err(serde::de::Error::custom(
                            format!("Invalid entity kind: {}. Acceptable values: auditor, badge, customer, project", s)
                        )),
                    })
                    .collect::<Result<Vec<_>, D::Error>>()?;
                
                if kinds.is_empty() {
                    return Err(serde::de::Error::custom("At least one entity kind must be specified"));
                }
                
                Ok(kinds)
            } else {
                let kind = match s.to_lowercase().as_str() {
                    "auditor" => Ok(EntityKind::Auditor),
                    "badge" => Ok(EntityKind::Badge),
                    "customer" => Ok(EntityKind::Customer),
                    "project" => Ok(EntityKind::Project),
                    _ => Err(serde::de::Error::custom(
                        format!("Invalid entity kind: {}. Acceptable values: auditor, badge, customer, project", s)
                    )),
                }?;
                Ok(vec![kind])
            }
        }
        None => Err(serde::de::Error::custom("Entity kind is required")),
    }
}

fn split_query(query: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for c in query.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            },
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    result.push(current);
                    current = String::new();
                }
            },
            _ => current.push(c),
        }
    }
    
    if !current.is_empty() {
        result.push(current);
    }
    
    result
}

fn parse_range_value(value: &str) -> (Option<String>, Option<String>) {
    let mut from = None;
    let mut to = None;
    
    if value.contains("..") {
        let mut range = value.splitn(2, "..");
        if let (Some(start), Some(end)) = (range.next(), range.next()) {
            if !start.is_empty() {
                from = Some(start.to_string());
            }
            if !end.is_empty() {
                to = Some(end.to_string());
            }
        }
    } else if value.starts_with('>') {
        let val = value[1..].trim();
        if !val.is_empty() {
            from = Some(val.to_string());
        }
    } else if value.starts_with('<') {
        let val = value[1..].trim();
        if !val.is_empty() {
            to = Some(val.to_string());
        }
    } else {
        if value.parse::<f64>().is_ok() {
            if value.contains("price") {
                to = Some(value.to_string());
            } 
            else if value.contains("rating") {
                from = Some(value.to_string());
            }
            else {
                to = Some(value.to_string());
            }
        }
    }
    
    (from, to)
}
