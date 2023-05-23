use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{access_rules::AccessRules, auth::Auth};

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
    pub fn apply(&self, action: &Action) -> Option<Status> {
        match (self, action) {
            (Status::Draft, Action::Begin) => todo!(),
            (Status::Draft, Action::Fixed) => todo!(),
            (Status::Draft, Action::NotFixed) => todo!(),
            (Status::Draft, Action::Discard) => todo!(),
            (Status::InProgress, Action::Begin) => todo!(),
            (Status::InProgress, Action::Fixed) => todo!(),
            (Status::InProgress, Action::NotFixed) => todo!(),
            (Status::InProgress, Action::Discard) => todo!(),
            (Status::Verification, Action::Begin) => todo!(),
            (Status::Verification, Action::Fixed) => todo!(),
            (Status::Verification, Action::NotFixed) => todo!(),
            (Status::Verification, Action::Discard) => todo!(),
            (Status::WillNotFix, Action::Begin) => todo!(),
            (Status::WillNotFix, Action::Fixed) => todo!(),
            (Status::WillNotFix, Action::NotFixed) => todo!(),
            (Status::WillNotFix, Action::Discard) => todo!(),
            (Status::Fixed, Action::Begin) => todo!(),
            (Status::Fixed, Action::Fixed) => todo!(),
            (Status::Fixed, Action::NotFixed) => todo!(),
            (Status::Fixed, Action::Discard) => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    Begin,
    Fixed,
    NotFixed,
    Discard,
}

impl Action {
    pub fn is_customer(&self) -> bool {
        match self {
            Action::Begin | Action::NotFixed => false,
            Action::Fixed |Action::Discard => true,
        }
    }

    pub fn is_auditor(&self) -> bool {
        match self {
            Action::Begin | Action::NotFixed |Action::Fixed => true,
            Action::Discard => false,
        }
    }
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
            link: self.link,
            status: self.status,
            include: self.include,
            feedback: self.feedback,
            events: Event::to_string_map(self.events),
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ChangeIssue {
    pub name: Option<String>,
    pub description: Option<String>,

    pub category: Option<String>,
    pub link: Option<String>,

    pub severity: Option<String>,

    pub status: Option<Action>,
    pub include: Option<bool>,

    pub feedback: Option<String>,
    pub events: Option<Vec<CreateEvent>>,
}

impl ChangeIssue {
    pub fn get_access_auditor(&self, _audit: &Audit<ObjectId>) -> bool {
        let status = if let Some(action) = &self.status {
            action.is_auditor()
        } else {
            true
        };
        status
    }

    pub fn get_access_customer(&self, _audit: &Audit<ObjectId>) -> bool {
        let status = if let Some(action) = &self.status {
            action.is_customer()
        } else {
            true
        };
        self.include.is_none() && status
    }
}

impl<'a, 'b> AccessRules<&'a Audit<ObjectId>, &'b Auth> for ChangeIssue {
    fn get_access(&self, object: &'a Audit<ObjectId>, subject: &'b Auth) -> bool {
        match subject {
            Auth::Service(_, _) => true,
            Auth::Admin(_) => true,
            Auth::User(id) => {
                if &object.auditor_id == id {
                    self.get_access_auditor(&object)
                } else if &object.customer_id == id {
                    self.get_access_customer(&object)
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
}

pub struct UpdateEvent {}
