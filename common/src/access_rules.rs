use mongodb::bson::oid::ObjectId;

use crate::{auth::Auth, entities::user::User};

pub trait AccessRules<Object, Subject> {
    fn get_access(object: Object, subject: Subject) -> bool;
}

pub struct Edit;

pub struct Read;

impl<'a, 'b> AccessRules<&'a Auth, &'b User<ObjectId>> for Edit {
    fn get_access(auth: &'a Auth, user: &'b User<ObjectId>) -> bool {
        if let Auth::User(id) = auth {
            &user.id == id
        } else {
            true
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b User<ObjectId>> for Read {
    fn get_access(auth: &'a Auth, user: &'b User<ObjectId>) -> bool {
        if let Auth::User(id) = auth {
            &user.id == id
        } else {
            true
        }
    }
}
