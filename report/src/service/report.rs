use actix_files::NamedFile;
use actix_web::HttpResponse;
use common::{
    api::audits::PublicAudit,
    context::Context,
    entities::user::PublicUser,
    services::{FILES_SERVICE, PROTOCOL, USERS_SERVICE},
};
use reqwest::{
    multipart::{Form, Part},
    Response,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RendererInput {
    pub auditor_name: String,
    pub auditor_email: String,
    pub project_name: String,
    pub scope: Vec<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicReport {
    path: String,
}

pub async fn create_report(context: Context, audit: PublicAudit) -> anyhow::Result<PublicReport> {
    let markdown = audit.issues.iter().fold(String::new(), |mut acc, issue| {
        acc.push_str(&format!("## {}\n\n{}\n\n", issue.name, issue.description));
        acc
    });

    let user = context
        .make_request()
        .get(format!(
            "{}://{}/api/user/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            audit.auditor_id
        ))
        .json(&audit.auditor_id)
        .send()
        .await
        .unwrap()
        .json::<PublicUser>()
        .await?;

    let input = RendererInput {
        auditor_name: audit.auditor_first_name + " " + &audit.auditor_last_name,
        auditor_email: user.email,
        project_name: audit.project_name,
        scope: audit.scope,
        markdown,
    };

    let report = context
        .make_request()
        .post(format!(
            "{}://{}/api/generate-report",
            PROTOCOL.as_str(),
            FILES_SERVICE.as_str()
        ))
        .json(&input)
        .send()
        .await
        .unwrap()
        .bytes()
        .await?;

    let path = audit.customer_id.clone() + &audit.auditor_id;

    let client = &context.0.client;
    let form = Form::new()
        .part("file", Part::bytes(report.to_vec()))
        .part("path", Part::text(path.clone()))
        .part("original_name", Part::text("report.pdf"))
        .part("private", Part::text("true"))
        .part("customerId", Part::text(audit.customer_id))
        .part("auditorId", Part::text(audit.auditor_id));

    let _ = client
        .post(format!(
            "{}://{}/api/file",
            PROTOCOL.as_str(),
            FILES_SERVICE.as_str()
        ))
        .multipart(form)
        .send()
        .await?;

    Ok(PublicReport { path })
}

/*

{
    "auditor_name": "Aleksander Masloww",
    "auditor_email": "maslow@gmail.com",
    "project_name": "THIS IS PROJECT NAME",
    "scope": [
        "https://google.com",
        "https://github.com"
    ],
    "markdown": "# header1\n---\n## header2\n---\n* **bold**\n* *italic*\n"
}

*/
