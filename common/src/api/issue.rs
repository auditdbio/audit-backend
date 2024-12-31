use serde::{Deserialize, Serialize};

use crate::{
    default_timestamp,
    entities::issue::{Event, Status},
};
use crate::entities::issue::IssueEditHistory;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicIssue {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub severity: String,

    pub category: String,
    #[serde(default)]
    pub links: Vec<String>,

    pub status: Status,
    pub include: bool,

    pub feedback: String,
    pub events: Vec<Event<String>>,
    #[serde(default = "default_timestamp")]
    pub last_modified: i64,
    pub read: u64,
    pub edit_history: Vec<IssueEditHistory>,
}
