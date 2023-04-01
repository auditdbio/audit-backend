use mongodb::bson::oid::ObjectId;

use crate::{
    auth_session::{AuthSession, Role},
    entities::{
        audit::Audit, audit_request::AuditRequest, auditor::Auditor, customer::Customer,
        project::Project,
    },
};

pub trait Ruleset<Subject, Object> {
    fn request_access(subject: &Subject, object: &Object) -> bool;
}

pub struct Read;

pub struct Edit;

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, AuditRequest<T>> for Read {
    fn request_access(subject: &AuthSession, object: &AuditRequest<T>) -> bool {
        subject.user_id == object.auditor_id.clone().into()
            || subject.user_id == object.customer_id.clone().into()
            || subject.role != Role::User
    }
}

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, Audit<T>> for Read {
    fn request_access(subject: &AuthSession, object: &Audit<T>) -> bool {
        subject.user_id == object.auditor_id.clone().into()
            || subject.user_id == object.customer_id.clone().into()
            || subject.role != Role::User
    }
}

impl<T> Ruleset<(), Auditor<T>> for Read {
    fn request_access(_subject: &(), object: &Auditor<T>) -> bool {
        true // auditors pages are now open for all site visitors
    }
}

impl<T> Ruleset<AuthSession, Customer<T>> for Read {
    fn request_access(subject: &AuthSession, _object: &Customer<T>) -> bool {
        true // customers pages are now open for all site visitors
    }
}

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, Project<T>> for Read {
    fn request_access(subject: &AuthSession, object: &Project<T>) -> bool {
        subject.user_id == object.customer_id.clone().into() || subject.role != Role::User
    }
}

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, AuditRequest<T>> for Edit {
    fn request_access(subject: &AuthSession, object: &AuditRequest<T>) -> bool {
        subject.user_id == object.auditor_id.clone().into()
            || subject.user_id == object.customer_id.clone().into()
            || subject.role != Role::User
    }
}

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, Audit<T>> for Edit {
    fn request_access(subject: &AuthSession, object: &Audit<T>) -> bool {
        subject.user_id == object.auditor_id.clone().into()
            || subject.user_id == object.customer_id.clone().into()
            || subject.role != Role::User
    }
}

impl<T> Ruleset<(), Auditor<T>> for Edit {
    fn request_access(_subject: &(), object: &Auditor<T>) -> bool {
        true // auditors pages are now open for all site visitors
    }
}

impl<T> Ruleset<AuthSession, Customer<T>> for Edit {
    fn request_access(subject: &AuthSession, _object: &Customer<T>) -> bool {
        true // customers pages are now open for all site visitors
    }
}

impl<T: Clone + Into<ObjectId>> Ruleset<AuthSession, Project<T>> for Edit {
    fn request_access(subject: &AuthSession, object: &Project<T>) -> bool {
        subject.user_id == object.customer_id.clone().into() || subject.role != Role::User
    }
}
