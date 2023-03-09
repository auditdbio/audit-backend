use common::{entities::user::User, ruleset::Ruleset};
use mongodb::bson::oid::ObjectId;

use crate::handlers::auth::LoginRequest;

pub struct Login;

impl Ruleset<&LoginRequest, &User<ObjectId>> for Login {
    fn request_access(subject: &LoginRequest, object: &User<ObjectId>) -> bool {
        subject.password == object.password
    }
}
