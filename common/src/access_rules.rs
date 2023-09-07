use mongodb::bson::oid::ObjectId;

use crate::{
    auth::Auth,
    entities::{
        audit::{Audit, AuditStatus},
        audit_request::AuditRequest,
        auditor::Auditor,
        bage::Bage,
        customer::Customer,
        project::Project,
        user::User,
    },
};

pub trait AccessRules<Object, Subject> {
    fn get_access(&self, object: Object, subject: Subject) -> bool;
}

pub struct Read;

pub struct Edit;

pub struct Delete;

impl<'a, 'b> AccessRules<&'a Auth, &'b User<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, _user: &'b User<ObjectId>) -> bool {
        #[allow(clippy::match_single_binding)]
        match auth {
            _ => true,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b User<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, user: &'b User<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) => true,
            Auth::Admin(_) => true,
            Auth::User(id) => id == &user.id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Customer<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, _customer: &'b Customer<ObjectId>) -> bool {
        #[allow(clippy::match_single_binding)]
        match auth {
            _ => true,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Customer<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, customer: &'b Customer<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => id == &customer.user_id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Auditor<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, _auditor: &'b Auditor<ObjectId>) -> bool {
        #[allow(clippy::match_single_binding)]
        match auth {
            _ => true,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Bage<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, _auditor: &'b Bage<ObjectId>) -> bool {
        #[allow(clippy::match_single_binding)]
        match auth {
            _ => true,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Auditor<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, auditor: &'b Auditor<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => id == &auditor.user_id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Bage<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, _bage: &'b Bage<ObjectId>) -> bool {
        matches!(auth, Auth::Service(_, _) | Auth::Admin(_))
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Project<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, project: &'b Project<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => {
                project.publish_options.publish
                    || id == &project.customer_id
                    || project.auditors.contains(id)
            }
            Auth::None => project.publish_options.publish,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Project<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, project: &'b Project<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => project.publish_options.publish || &project.customer_id == id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b AuditRequest<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, request: &'b AuditRequest<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => &request.customer_id == id || &request.auditor_id == id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b AuditRequest<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, request: &'b AuditRequest<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => &request.customer_id == id || &request.auditor_id == id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Audit<ObjectId>> for Read {
    fn get_access(&self, auth: &'a Auth, request: &'b Audit<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => &request.customer_id == id || &request.auditor_id == id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Audit<ObjectId>> for Edit {
    fn get_access(&self, auth: &'a Auth, request: &'b Audit<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => &request.customer_id == id || &request.auditor_id == id,
            Auth::None => false,
        }
    }
}

impl<'a, 'b> AccessRules<&'a Auth, &'b Audit<ObjectId>> for Delete {
    fn get_access(&self, auth: &'a Auth, audit: &'b Audit<ObjectId>) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(id) => {
                (&audit.customer_id == id && audit.status == AuditStatus::Waiting)
                    || &audit.auditor_id == id
            }
            Auth::None => false,
        }
    }
}

pub struct GetData;

impl<'a> AccessRules<&'a Auth, ()> for GetData {
    fn get_access(&self, auth: &'a Auth, _user: ()) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(_) => false,
            Auth::None => false,
        }
    }
}

pub struct SendMail;

impl<'a> AccessRules<&'a Auth, ()> for SendMail {
    fn get_access(&self, auth: &'a Auth, _user: ()) -> bool {
        match auth {
            Auth::Service(_, _) | Auth::Admin(_) => true,
            Auth::User(_) => false,
            Auth::None => false,
        }
    }
}
