use common::{entities::user::User, ruleset::Ruleset};

use crate::handlers::auth::LoginRequest;

pub struct Login;

impl Ruleset<&LoginRequest, &User> for Login {
    fn request_access(subject: &LoginRequest, object: &User) -> bool {
        subject.password == object.password
    }
}
