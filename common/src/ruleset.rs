pub trait Ruleset<Subject,Object> {
    fn request_access(subject:Subject, object:Object) -> bool;
}


struct EditProject;
struct InviteAuditor;
struct AcceptInvitation;