use actix_web::{post, web::Json};
use serde::{Deserialize, Serialize};

use crate::service::report::create_pandoc_report;

#[derive(Serialize, Deserialize)]
pub struct CreateReport {
    pub markdown: String,
}

#[derive(Serialize, Deserialize)]
pub struct Report {
    pub latex: String,
}

#[post("/api/report")]
pub async fn create_report(Json(CreateReport { markdown }): Json<CreateReport>) -> Json<Report> {
    let report = create_pandoc_report(markdown).await.unwrap();
    Json(Report { latex: report })
}
