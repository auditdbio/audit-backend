use actix_web::{get, web, HttpRequest, HttpResponse};
use common::{
    auth_session::{AuthSessionManager, Role, SessionManager},
    entities::{customer::Customer, project::Project},
};
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::repositories::{customer::CustomerRepo, project::ProjectRepo};

pub mod customers;
pub mod projects;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerServiceResponse {
    pub projects: Vec<Document>,
    pub customers: Vec<Document>,
}

#[get("/api/customer/data/{resource}/{timestamp}")]
pub async fn get_data(
    req: HttpRequest,
    since: web::Path<(String, i64)>,
    project_repo: web::Data<ProjectRepo>,
    customer_repo: web::Data<CustomerRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let (resource, since) = since.into_inner();
    //let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap
    // if session.role != Role::Service {
    //     return Ok(HttpResponse::Unauthorized().finish());
    // }

    match resource.as_str() {
        "project" => {
            let projects = project_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                projects
                    .into_iter()
                    .map(Project::stringify)
                    .map(Project::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        "customer" => {
            let customers = customer_repo.get_all_since(since).await?;
            Ok(HttpResponse::Ok().json(
                customers
                    .into_iter()
                    .map(Customer::stringify)
                    .map(Customer::to_doc)
                    .collect::<Vec<_>>(),
            ))
        }
        _ => Ok(HttpResponse::NotFound().finish()),
    }
}
