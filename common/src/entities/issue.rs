use std::{collections::HashMap, hash::Hash};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    access_rules::AccessRules,
    auth::Auth,
    entities::role::Role,
    default_timestamp,
};

use super::audit::Audit;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Draft,
    InProgress,
    Verification,
    WillNotFix,
    Fixed,
}

impl Status {
    pub fn apply(&self, action: &Action, role: Role) -> Option<Status> {
        match (self, action, role) {
            (Status::Draft, Action::Begin, Role::Auditor) => Some(Status::InProgress),

            (Status::InProgress, Action::Begin, Role::Auditor) => Some(Status::Draft),
            (Status::InProgress, Action::Fixed, Role::Auditor) => Some(Status::Fixed),
            (Status::InProgress, Action::Fixed, Role::Customer) => Some(Status::Verification),
            (Status::InProgress, Action::Discard, _) => Some(Status::WillNotFix),

            (Status::Verification, Action::NotFixed, _) => Some(Status::InProgress),
            (Status::Verification, Action::Discard, _) => Some(Status::WillNotFix),
            (Status::Verification, Action::Verified, Role::Auditor) => Some(Status::Fixed),

            (Status::WillNotFix, Action::Fixed, Role::Auditor) => Some(Status::Fixed),
            (Status::WillNotFix, Action::Verified, Role::Auditor) => Some(Status::Verification),
            (Status::WillNotFix, Action::ReOpen, _) => Some(Status::InProgress),

            (Status::Fixed, Action::Discard, Role::Auditor) => Some(Status::WillNotFix),
            (Status::Fixed, Action::NotFixed, Role::Auditor) => Some(Status::Verification),
            (Status::Fixed, Action::ReOpen, Role::Auditor) => Some(Status::InProgress),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    Begin,
    Fixed,
    Verified,
    NotFixed,
    Discard,
    ReOpen,
}

// impl Action {
//     pub fn is_customer(&self) -> bool {
//         match self {
//             Action::Begin | Action::NotFixed | Action::Verified => false,
//             Action::Fixed | Action::Discard | Action::ReOpen => true,
//         }
//     }
// }

pub fn severity_to_integer(severity: &str) -> usize {
    match severity {
        "Critical" | "critical" => 0,
        "Major" | "major" => 1,
        "Medium" | "medium" => 2,
        "Minor" | "minor" => 3,
        _ => 4,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Issue<Id> {
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
    pub events: Vec<Event<Id>>,
    #[serde(default = "default_timestamp")]
    pub last_modified: i64,
    #[serde(default)]
    pub read: HashMap<String, u64>,
}

impl<T> Issue<T> {
    pub fn is_resolved(&self) -> bool {
        !self.include || self.status == Status::Fixed || self.status == Status::WillNotFix
    }
}

impl Issue<String> {
    pub fn parse(self) -> Issue<ObjectId> {
        Issue {
            id: self.id,
            name: self.name,
            description: self.description,
            severity: self.severity,
            category: self.category,
            links: self.links,
            status: self.status,
            include: self.include,
            feedback: self.feedback,
            events: Event::parse_map(self.events),
            last_modified: self.last_modified,
            read: self.read,
        }
    }

    pub fn parse_map(map: Vec<Self>) -> Vec<Issue<ObjectId>> {
        map.into_iter().map(|v| v.parse()).collect()
    }
}

impl Issue<ObjectId> {
    pub fn to_string(self) -> Issue<String> {
        Issue {
            id: self.id,
            name: self.name,
            description: self.description,
            severity: self.severity,
            category: self.category,
            links: self.links,
            status: self.status,
            include: self.include,
            feedback: self.feedback,
            events: Event::to_string_map(self.events),
            last_modified: self.last_modified,
            read: self.read,
        }
    }

    pub fn to_string_map(map: Vec<Self>) -> Vec<Issue<String>> {
        map.into_iter().map(|v| v.to_string()).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateEvent {
    pub kind: EventKind,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ChangeIssue {
    pub name: Option<String>,
    pub description: Option<String>,

    pub category: Option<String>,
    pub links: Option<Vec<String>>,

    pub severity: Option<String>,

    pub status: Option<Action>,
    pub include: Option<bool>,

    pub feedback: Option<String>,
    pub events: Option<Vec<CreateEvent>>,
}

impl ChangeIssue {
    pub fn get_access_auditor(&self, _audit: &Audit<ObjectId>) -> bool {
        true
    }

    pub fn get_access_customer(&self, _audit: &Audit<ObjectId>) -> bool {
        self.include.is_none()
    }
}

impl<'a, 'b> AccessRules<&'a Audit<ObjectId>, &'b Auth> for ChangeIssue {
    fn get_access(&self, object: &'a Audit<ObjectId>, subject: &'b Auth) -> bool {
        match subject {
            Auth::Service(_, _) => true,
            Auth::Admin(_) => true,
            Auth::User(id) => {
                if &object.auditor_id == id {
                    self.get_access_auditor(object)
                } else if &object.customer_id == id {
                    self.get_access_customer(object)
                } else {
                    false
                }
            }
            Auth::None => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event<Id> {
    pub id: usize,
    pub timestamp: i64,
    pub user: Id,
    pub kind: EventKind,
    pub message: String,
}

impl Event<String> {
    pub fn parse(self) -> Event<ObjectId> {
        Event {
            id: self.id,
            timestamp: self.timestamp,
            user: self.user.parse().unwrap(),
            kind: self.kind,
            message: self.message,
        }
    }

    pub fn parse_map(map: Vec<Self>) -> Vec<Event<ObjectId>> {
        map.into_iter().map(|v| v.parse()).collect()
    }
}

impl Event<ObjectId> {
    pub fn to_string(self) -> Event<String> {
        Event {
            id: self.id,
            timestamp: self.timestamp,
            user: self.user.to_string(),
            kind: self.kind,
            message: self.message,
        }
    }

    pub fn to_string_map(map: Vec<Self>) -> Vec<Event<String>> {
        map.into_iter().map(|v| v.to_string()).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EventKind {
    IssueName,
    IssueDescription,
    IssueSeverity,
    IssueCategory,
    IssueLink,
    StatusChange,
    Comment,
    FeedbackAdded,
    FeedbackChanged,
}

pub struct UpdateEvent {}
