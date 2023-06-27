use actix_multipart::Multipart;
use actix_web::{post, web::Json};
use futures::StreamExt;
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
pub async fn create_report(mut payload: Multipart) -> Json<Report> {
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        if field.name() == "markdown" {
            let mut md = vec![];
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                md.push(data);
            }
            let report = create_pandoc_report(String::from_utf8(md.concat()).unwrap())
                .await
                .unwrap();
            return Json(Report { latex: report });
        }
    }
    let report = String::new();
    Json(Report { latex: report })
}
