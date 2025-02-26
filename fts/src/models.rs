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
    #[serde(flatten)]
    pub price_range_params: PriceRangeParams,
    #[serde(flatten)]
    pub rating_params: RatingParams,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub partial_match: Option<bool>,
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct TagsVisitor;

    impl<'de> serde::de::Visitor<'de> for TagsVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(vec![value.to_string()]))
        }

        fn visit_seq<S>(self, mut visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut values = Vec::new();
            while let Some(value) = visitor.next_element()? {
                values.push(value);
            }
            Ok(Some(values))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(TagsVisitor)
}

#[derive(Debug, Deserialize, Default)]
pub struct PriceRangeParams {
    #[serde(rename = "price_range.from")]
    #[serde(deserialize_with = "deserialize_option_i64")]
    pub from: Option<i64>,
    #[serde(rename = "price_range.to")]
    #[serde(deserialize_with = "deserialize_option_i64")]
    pub to: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RatingParams {
    #[serde(rename = "rating.from")]
    #[serde(deserialize_with = "deserialize_option_f32")]
    pub from: Option<f32>,
    #[serde(rename = "rating.to")]
    #[serde(deserialize_with = "deserialize_option_f32")]
    pub to: Option<f32>,
}

fn deserialize_option_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            match s.parse::<i64>() {
                Ok(i) => Ok(Some(i)),
                Err(_) => Err(serde::de::Error::custom(format!("Invalid i64: {}", s))),
            }
        }
        None => Ok(None),
    }
}

fn deserialize_option_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            match s.parse::<f32>() {
                Ok(f) => Ok(Some(f)),
                Err(_) => Err(serde::de::Error::custom(format!("Invalid f32: {}", s))),
            }
        }
        None => Ok(None),
    }
}

impl SearchQuery {
    pub fn price_range(&self) -> Option<PriceRangeFilter> {
        if self.price_range_params.from.is_some() || self.price_range_params.to.is_some() {
            Some(PriceRangeFilter {
                from: self.price_range_params.from,
                to: self.price_range_params.to,
            })
        } else {
            None
        }
    }

    pub fn rating(&self) -> Option<RangeFilter<f32>> {
        if self.rating_params.from.is_some() || self.rating_params.to.is_some() {
            Some(RangeFilter {
                from: self.rating_params.from,
                to: self.rating_params.to,
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
    pub auditors: Vec<Auditor>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}