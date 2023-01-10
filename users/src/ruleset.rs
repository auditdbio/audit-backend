use common::ruleset::Ruleset;

use crate::{repositories::user::UserModel, handlers::auth::LoginRequest};

pub struct Login;

impl Ruleset<&LoginRequest, &UserModel> for Login {
    fn request_access(subject: &LoginRequest, object: &UserModel) -> bool {
        subject.password == object.password
    }
}
