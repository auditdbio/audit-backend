use chrono::Duration;

lazy_static::lazy_static! {
    pub static ref DURATION: Duration = Duration::days(1);
}
