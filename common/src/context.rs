use std::sync::Arc;

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use serde::Serialize;
use type_map::concurrent::TypeMap;

use crate::{
    auth::Auth,
    error::{self, ServiceError},
    repository::{Repository, RepositoryObject},
};

pub struct ServiceState {
    pub repositories: TypeMap,
    pub client: reqwest::Client,
    pub service_auth: Auth,
}

impl ServiceState {
    pub fn new(service_name: String) -> Self {
        Self {
            repositories: TypeMap::new(),
            client: reqwest::Client::new(),
            service_auth: Auth::Service(service_name),
        }
    }

    pub fn insert<T>(&mut self, repository: impl Repository<T> + Send + Sync + 'static) {
        self.repositories.insert(repository);
    }
}

pub struct HandlerContext {
    pub user_auth: Option<Auth>,
}

pub struct Context(pub Arc<ServiceState>, pub HandlerContext);

impl FromRequest for Context {
    type Error = ServiceError;

    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        fn from_request_inner(req: &HttpRequest, _payload: &mut Payload) -> error::Result<Context> {
            let auth = req
                .headers()
                .get("Authorization")
                .and_then(|x| x.to_str().ok())
                .and_then(|x| x.strip_prefix("Bearer ")) // remove prefix
                .map(Auth::from_token);

            let user_auth = if let Some(auth) = auth {
                Some(auth?)
            } else {
                None
            };


            let Some(state) = req
                .app_data::<Arc<ServiceState>>()else {
                    return Err(anyhow::anyhow!("No state provided".to_string()).into());
                };

            Ok(Context(
                Arc::clone(state),
                HandlerContext {
                    user_auth,
                },
            ))
        }
        let result = from_request_inner(req, payload);

        Box::pin(async move { result })
    }
}

pub struct ServiceRequest<'a, 'b, T = ()> {
    // TODO: return error, if we try to send body into GET request
    client: &'a reqwest::Client,
    method: reqwest::Method,
    url: Option<String>,
    body: Option<&'b T>,
    auth: Auth,
}

impl<'a, 'b, T: Serialize> ServiceRequest<'a, 'b, T> {
    pub fn new(client: &'a reqwest::Client, auth: Auth) -> Self {
        Self {
            client,
            auth,
            method: reqwest::Method::GET,
            url: None,
            body: None,
        }
    }

    pub fn get(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn post(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::POST;
        self
    }

    pub fn patch(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::PATCH;
        self
    }

    pub fn delete(mut self, url: String) -> Self {
        self.url = Some(url);
        self.method = reqwest::Method::DELETE;
        self
    }

    pub fn json(mut self, body: &'b T) -> Self {
        self.body = Some(body);
        self
    }

    pub async fn send(self) -> anyhow::Result<reqwest::Response> {
        let url = self.url.as_ref().unwrap();
        let mut request = self.client.request(self.method, url);
        if let Some(body) = self.body {
            request = request.json(body);
        }
        let response = request.send().await?;
        Ok(response)
    }

    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = auth;
        self
    }
}

impl Context {
    pub fn get_repository<T: 'static>(&self) -> Option<RepositoryObject<T>> {
        self.0.repositories.get::<RepositoryObject<T>>().cloned()
    }

    pub fn make_request<T: Serialize>(&self) -> ServiceRequest<T> {
        ServiceRequest::<T>::new(&self.0.client, self.0.service_auth.clone())
    }
}

pub struct MutationContext<'a> {
    pub context: &'a Context,
    pub current_field: Option<String>,
}

impl<'a> MutationContext<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            current_field: None,
            context,
        }
    }
}
