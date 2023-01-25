pub trait Ruleset<Subject, Object> {
    fn request_access(subject: Subject, object: Object) -> bool;
}
