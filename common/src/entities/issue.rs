use std::collections::HashMap;
use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Draft,
    InProgress,
    Verification,
    WillNotFix,
    Fixed,
}

pub enum Action {
    Begin,
    Fixed,
    NotFixed,
    Discard,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Issue<Id> {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub severity: String,

    pub category: String,
    pub link: String,

    pub status: Status,
    pub include: bool,

    pub feedback: String,
    pub events: Vec<Event<Id>>,
}

impl Issue<String> {
    pub fn parse(self) -> Issue<ObjectId> {
        Issue {
            id: self.id,
            name: self.name,
            description: self.description,
            severity: self.severity,
            category: self.category,
            link: self.link,
            status: self.status,
            include: self.include,
            feedback: self.feedback,
            events: Event::parse_map(self.events),
        }
    }

    pub fn parse_map(map: HashMap<String, Self>) -> HashMap<ObjectId, Issue<ObjectId>> {
        map.into_iter()
            .map(|(k, v)| (k.parse().unwrap(), v.parse()))
            .collect()
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
            link: self.link,
            status: self.status,
            include: self.include,
            feedback: self.feedback,
            events: Event::to_string_map(self.events),
        }
    }

    pub fn to_string_map(map: HashMap<ObjectId, Self>) -> HashMap<String, Issue<String>> {
        map.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}

pub struct IssueUpdate {
    pub name: Option<String>,
    pub description: Option<String>,

    pub category: Option<String>,
    pub link: Option<String>,

    pub status: Option<Action>,
    include: Option<bool>,

    feedback: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event<Id> {
    id: usize,
    timestamp: i64,
    user: Id,
    kind: EventKind,
    message: String,
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
        map.into_iter()
            .map(|v| v.parse())
            .collect()
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
        map.into_iter()
            .map(|v| v.to_string())
            .collect()
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
}

pub struct UpdateEvent {}
