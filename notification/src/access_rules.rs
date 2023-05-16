use common::{access_rules::AccessRules, auth::Auth};
use mongodb::bson::oid::ObjectId;

pub struct SendNotification;

impl<'a, 'b> AccessRules<&'a Auth, &'b ObjectId> for SendNotification {
    fn get_access(auth: &'a Auth, user_id: &'b ObjectId) -> bool {
        match auth {
            Auth::Service(_) | Auth::Admin(_) => true,
            Auth::User(id) => id == user_id,
            _ => false,
        }
    }
}
