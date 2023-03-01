use std::collections::HashMap;

use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpRequest, HttpResponse,
};
use common::{
    auth_session::{AuthSessionManager, SessionManager},
    entities::customer::Customer,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::Result, repositories::customer::CustomerRepo};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostCustomerRequest {
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    contacts: HashMap<String, String>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PostCustomerRequest
    ),
    responses(
        (status = 200)
    )
)]
#[post("/api/customer")]
pub async fn post_customer(
    req: HttpRequest,
    Json(data): web::Json<PostCustomerRequest>,
    repo: web::Data<CustomerRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let customer = Customer {
        user_id: session.user_id(),
        first_name: data.first_name,
        last_name: data.last_name,
        about: data.about,
        company: data.company,
        contacts: data.contacts,
    };

    if !repo.create(&customer).await? {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: customer entity already exits
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    responses(
        (status = 200, body = Customer)
    )
)]
#[get("/api/customer")]
pub async fn get_customer(
    req: HttpRequest,
    repo: web::Data<CustomerRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(customer) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(customer))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatchCustomerRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<HashMap<String, String>>,
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = PatchCustomerRequest
    ),
    responses(
        (status = 200, body = Customer)
    )
)]
#[patch("/api/customer")]
pub async fn patch_customer(
    req: HttpRequest,
    web::Json(data): web::Json<PatchCustomerRequest>,
    repo: web::Data<CustomerRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(mut customer) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if let Some(first_name) = data.first_name {
        customer.first_name = first_name;
    }

    if let Some(last_name) = data.last_name {
        customer.last_name = last_name;
    }

    if let Some(about) = data.about {
        customer.about = about;
    }

    if let Some(company) = data.company {
        customer.company = company;
    }

    if let Some(contacts) = data.contacts {
        customer.contacts = contacts;
    }

    repo.delete(&session.user_id()).await.unwrap();
    repo.create(&customer).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    params(
        ("Authorization" = String, Header,  description = "Bearer token"),
    ),
    request_body(
        content = CustomerRepository
    ),
    responses(
        (status = 200, body = Customer)
    )
)]
#[delete("/api/customer")]
pub async fn delete_customer(
    req: HttpRequest,
    repo: web::Data<CustomerRepo>,
    manager: web::Data<AuthSessionManager>,
) -> Result<HttpResponse> {
    let session = manager.get_session(req.into()).await.unwrap(); // TODO: remove unwrap

    let Some(customer) = repo.delete(&session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: this user doesn't exit
    };
    Ok(HttpResponse::Ok().json(customer))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::test::{self, init_service};
    use common::auth_session::AuthSession;
    use mongodb::bson::oid::ObjectId;

    use crate::{create_test_app, PatchCustomerRequest, PostCustomerRequest};

    #[actix_web::test]
    async fn test_post_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::post()
            .uri("/api/customer")
            .set_json(&PostCustomerRequest {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                about: "I'm a test".to_string(),
                company: "Test Inc.".to_string(),
                contacts: HashMap::new(),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_patch_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::patch()
            .uri("/api/customer")
            .set_json(&PatchCustomerRequest {
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                about: Some("I'm a test".to_string()),
                company: Some("Test Inc.".to_string()),
                contacts: Some(HashMap::new()),
            })
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_delete_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::delete()
            .uri("/api/customer")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_customer() {
        let test_user = AuthSession {
            user_id: ObjectId::new(),
            token: "".to_string(),
            exp: 100000000,
        };

        let app = init_service(create_test_app(test_user)).await;
        let req = actix_web::test::TestRequest::get()
            .uri("/api/customer")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());
    }
}
