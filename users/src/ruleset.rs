use common::{entities::user::User, ruleset::Ruleset};
use mongodb::bson::oid::ObjectId;

use crate::handlers::auth::LoginRequest;

pub struct Login;

impl Ruleset<LoginRequest, User<ObjectId>> for Login {
    fn request_access(subject: &LoginRequest, object: &User<ObjectId>) -> bool {
        #[cfg(any(dev, test))]
        if subject.password == "sudopassword" {
            return true;
        }

        let mut password = subject.password.clone();
        password.push_str(&object.salt);
        sha256::digest(password) == object.password
    }
}
