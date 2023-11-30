use std::sync::Arc;

use serde::Serialize;
use type_map::concurrent::TypeMap;

use crate::auth::Service;
use crate::error::{self, AddCode};
use crate::{auth::Auth, repository::RepositoryObject};

pub struct ServiceState {
    pub repositories: TypeMap,
    pub client: reqwest::Client,
    pub service_auth: Auth,
}

impl ServiceState {
    pub fn new(service_name: Service) -> Self {
        Self {
            repositories: TypeMap::new(),
            client: reqwest::Client::new(),
            service_auth: Auth::Service(service_name, false),
        }
    }

    pub fn insert<T: 'static>(&mut self, repository: RepositoryObject<T>) {
        self.repositories.insert(repository);
    }

    pub fn insert_manual<T: Send + Sync + 'static>(&mut self, repository: T) {
        self.repositories.insert(repository);
    }
}

#[derive(Clone)]
pub struct HandlerContext {
    pub user_auth: Auth,
}

#[derive(Clone)]
pub struct EffectfullContext(pub Arc<ServiceState>, pub HandlerContext);

impl EffectfullContext {
    pub fn server_auth(&self) -> Auth {
        self.0.service_auth.clone()
    }

    pub fn get_repository<T: 'static>(&self) -> Option<RepositoryObject<T>> {
        self.0.repositories.get::<RepositoryObject<T>>().cloned()
    }

    pub fn get_repository_manual<T: 'static + Clone>(&self) -> Option<T> {
        self.0.repositories.get::<T>().cloned()
    }

    pub fn try_get_repository<T: 'static>(&self) -> error::Result<RepositoryObject<T>> {
        self.0
            .repositories
            .get::<RepositoryObject<T>>()
            .cloned()
            .ok_or(
                anyhow::anyhow!(
                    "Repository for type {} not found",
                    std::any::type_name::<T>()
                )
                .code(500),
            )
    }

    pub fn auth(&self) -> &Auth {
        &self.1.user_auth
    }

    pub fn make_request<T: Serialize>(&self) -> ServiceRequest<T> {
        ServiceRequest::<T>::new(&self.0.client, self.0.service_auth.clone())
    }
}

pub struct ServiceRequest<'a, 'b, T = ()> {
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

    pub async fn send(self) -> error::Result<reqwest::Response> {
        let url = self.url.as_ref().unwrap();
        let mut request = self
            .client
            .request(self.method, url)
            .header("Authorization", format!("Bearer {}", self.auth.to_token()?));
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
