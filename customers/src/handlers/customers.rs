use std::{collections::HashMap, str::FromStr};

use actix_web::{HttpRequest, HttpResponse, post, patch, delete, get, web::{self, Json}};
use common::get_auth_session;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{error::Result, repositories::customer::{CustomerRepository, CustomerModel}};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostCustomerRequest {
    first_name: String,
    last_name: String,
    about: String,
    company: String,
    contacts: HashMap<String, String>,
}

#[post("/api/customer")]
pub async fn post_customer(req: HttpRequest, Json(data): web::Json<PostCustomerRequest>, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let customer = CustomerModel {
        user_id: session.user_id(), // TODO: remove unwrap
        first_name: data.first_name,
        last_name: data.last_name,
        about: data.about,
        company: data.company,
        contacts: data.contacts,
    };

    if !repo.create(customer).await? {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: customer entity already exits
    }

    Ok(HttpResponse::Ok().finish())
}

#[get("/api/customer")]
pub async fn get_customer(req: HttpRequest, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(customer) = repo.find(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    Ok(HttpResponse::Ok().json(customer))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchCustomerRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    about: Option<String>,
    company: Option<String>,
    contacts: Option<HashMap<String, String>>,
}

#[patch("/api/customer")]
pub async fn patch_customer(req: HttpRequest, web::Json(data): web::Json<PatchCustomerRequest>, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

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

    repo.delete(session.user_id()).await.unwrap();
    repo.create(customer).await?;

    Ok(HttpResponse::Ok().finish())
}

#[delete("/api/customer")]
pub async fn delete_customer(req: HttpRequest, repo: web::Data<CustomerRepository>) -> Result<HttpResponse> {
    let session = get_auth_session(&req).await.unwrap(); // TODO: remove unwrap

    let Some(customer) = repo.delete(session.user_id()).await? else {
        return Ok(HttpResponse::BadRequest().finish()); // TODO: Error: this user doesn't exit
    };
    Ok(HttpResponse::Ok().json(customer))
}


