use common::{entities::audit::Audit, access_rules::AccessRules, auth::Auth};
use mongodb::bson::oid::ObjectId;

pub struct Issue {
    pub id: ObjectId,
    pub name: String,
    pub description: String,

    pub category: String,
    pub link: String,

    pub status: String,
    include: bool,

    feedback: String,

}

pub struct IssueUpdate {
    pub name: Option<String>,
    pub description: Option<String>,

    pub category: Option<String>,
    pub link: Option<String>,

    pub status: Option<String>,
    include: Option<bool>,

    feedback: Option<String>,
    events: Vec<Event>,
}

pub struct Event {

}

pub struct ChangeInAudit<'a> (&'a Audit<ObjectId>);

impl<'a, 'b, 'c> AccessRules<&'a ObjectId, &'b Auth> for ChangeInAudit<'c> {
    fn get_access(object: &'a ObjectId, subject: &'b Auth) -> bool {
        todo!()
    }
}