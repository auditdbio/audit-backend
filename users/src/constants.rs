use chrono::Duration;

lazy_static::lazy_static! {
    pub static ref MAX_DURATION: Duration = Duration::days(1);
}
