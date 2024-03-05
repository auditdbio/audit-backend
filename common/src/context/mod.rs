use std::sync::Arc;

use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use anyhow::anyhow;
use serde::Serialize;

use crate::context::effectfull_context::{EffectfullContext, HandlerContext, ServiceState};
use crate::{
    auth::Auth,
    error::{self, AddCode, ServiceError},
    repository::RepositoryObject,
};

pub use self::context_trait::GeneralContext;
use self::effectfull_context::ServiceRequest;
pub mod context_trait;
pub mod effectfull_context;
pub mod test_context;

impl FromRequest for GeneralContext {
    type Error = ServiceError;

    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        fn from_request_inner(
            req: &HttpRequest,
            _payload: &mut Payload,
        ) -> error::Result<GeneralContext> {
            let auth = req
                .headers()
                .get("Authorization")
                .and_then(|x| x.to_str().ok())
                .and_then(|x| x.strip_prefix("Bearer ")) // remove prefix
                .map(Auth::from_token);

            let user_auth = match auth {
                Some(Ok(Some(res))) => {
                    log::info!("Token parsed successfully");
                    res
                }
                Some(Ok(None)) => {
                    log::error!("Token expired");
                    Auth::None
                }
                Some(err) => {
                    log::error!("Error parsing token: {:?}", err);
                    Auth::None
                }
                None => {
                    log::error!("No header provided");
                    Auth::None
                }
            };

            let Some(state) = req.app_data::<Data<Arc<ServiceState>>>() else {
                return Err(anyhow::anyhow!("No state provided".to_string()).into());
            };

            Ok(GeneralContext::Effectfull(EffectfullContext(
                Arc::clone(state),
                HandlerContext { user_auth },
            )))
        }
        let result = from_request_inner(req, payload);

        Box::pin(async move { result })
    }
}

impl GeneralContext {
    pub fn server_auth(&self) -> Auth {
        match self {
            GeneralContext::Effectfull(context) => context.0.service_auth,
            GeneralContext::Test(context) => context.service_auth,
        }
    }

    pub fn get_repository<T: 'static>(&self) -> Option<RepositoryObject<T>> {
        match self {
            GeneralContext::Effectfull(context) => {
                context.0.repositories.get::<RepositoryObject<T>>().cloned()
            }
            GeneralContext::Test(_context) => {
                panic!("This api will be depricated and should not be used for test context")
            }
        }
    }

    pub fn get_repository_manual<T: 'static + Clone>(&self) -> Option<T> {
        match self {
            GeneralContext::Effectfull(context) => context.0.repositories.get::<T>().cloned(),
            GeneralContext::Test(_context) => {
                panic!("This api will be depricated and should not be used for test context")
            }
        }
    }

    pub fn try_get_repository<T: 'static>(&self) -> error::Result<RepositoryObject<T>> {
        match self {
            GeneralContext::Effectfull(context) => context
                .0
                .repositories
                .get::<RepositoryObject<T>>()
                .cloned()
                .ok_or(
                    anyhow!(
                        "Repository for type {} not found",
                        std::any::type_name::<T>()
                    )
                    .code(500),
                ),
            GeneralContext::Test(_context) => {
                panic!("This api will be depricated and should not be used for test context")
            }
        }
    }

    pub fn auth(&self) -> Auth {
        match self {
            GeneralContext::Effectfull(context) => context.1.user_auth,
            GeneralContext::Test(context) => context.user_auth,
        }
    }

    pub fn make_request<T: Serialize>(&self) -> ServiceRequest<T> {
        match self {
            GeneralContext::Effectfull(context) => {
                ServiceRequest::<T>::new(&context.0.client, context.0.service_auth)
            }
            GeneralContext::Test(_context) => {
                panic!("This api will be depricated and should not be used for test context")
            }
        }
    }

    pub fn client(&self) -> &reqwest::Client {
        match self {
            GeneralContext::Effectfull(context) => &context.0.client,
            GeneralContext::Test(_context) => {
                panic!("This api will be depricated and should not be used for test context")
            }
        }
    }
}
