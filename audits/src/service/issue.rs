use common::{access_rules::AccessRules, auth::Auth, entities::audit::Audit};
use mongodb::bson::oid::ObjectId;

pub struct Issue<Id> {
    pub id: Id,
    pub name: String,
    pub description: String,

    pub category: String,
    pub link: String,

    pub status: String,
    include: bool,

    feedback: String,
    event: Vec<Event<Id>>,
}

pub struct IssueUpdate {
    pub name: Option<String>,
    pub description: Option<String>,

    pub category: Option<String>,
    pub link: Option<String>,

    pub status: Option<String>,
    include: Option<bool>,

    feedback: Option<String>,
}

pub struct Event<Id> {
    timestamp: i64,
    user: Id,
    kind: EventKind,
    message: String,
}

pub enum EventKind {}

pub struct UpdateEvent {}

pub struct ChangeInAudit<'a>(&'a Audit<ObjectId>);

impl<'a, 'b, 'c> AccessRules<&'a ObjectId, &'b Auth> for ChangeInAudit<'c> {
    fn get_access(object: &'a ObjectId, subject: &'b Auth) -> bool {
        todo!()
    }
}
