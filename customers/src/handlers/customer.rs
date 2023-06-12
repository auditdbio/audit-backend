use actix_web::{
    delete, get, patch, post,
    web::{self, Json},
    HttpResponse,
};

use common::{
    context::Context,
    entities::{
        customer::{Customer, PublicCustomer},
        project::PublicProject,
    },
    error,
};

use serde_json::json;

use crate::service::customer::{CreateCustomer, CustomerChange, CustomerService};

#[post("/api/customer")]
pub async fn post_customer(
    context: Context,
    Json(data): web::Json<CreateCustomer>,
) -> error::Result<Json<Customer<String>>> {
    Ok(Json(CustomerService::new(context).create(data).await?))
}

#[get("/api/customer/{id}")]
pub async fn get_customer(context: Context, id: web::Path<String>) -> error::Result<HttpResponse> {
    let res = CustomerService::new(context).find(id.parse()?).await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[get("/api/my_customer")]
pub async fn my_customer(context: Context) -> error::Result<HttpResponse> {
    let res = CustomerService::new(context).my_customer().await?;
    if let Some(res) = res {
        Ok(HttpResponse::Ok().json(res))
    } else {
        Ok(HttpResponse::Ok().json(json! {{}}))
    }
}

#[patch("/api/my_customer")]
pub async fn patch_customer(
    context: Context,
    Json(data): Json<CustomerChange>,
) -> error::Result<Json<Customer<String>>> {
    Ok(Json(CustomerService::new(context).change(data).await?))
}

#[delete("/api/customer/{id}")]
pub async fn delete_customer(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<PublicCustomer>> {
    Ok(Json(
        CustomerService::new(context).delete(id.parse()?).await?,
    ))
}

#[get("/api/customer/project/{id}")]
pub async fn get_customer_projects(
    context: Context,
    id: web::Path<String>,
) -> error::Result<Json<Vec<PublicProject>>> {
    Ok(Json(
        CustomerService::new(context)
            .get_projects(id.parse()?)
            .await?,
    ))
}
