use crate::auth::Auth;

#[derive(Debug, Clone)]
pub struct TestContext {
    pub service_auth: Auth,
    pub user_auth: Auth,
}
