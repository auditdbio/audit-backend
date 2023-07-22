use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

use common::{
    api::{
        audits::{AuditChange, PublicAudit},
        issue::PublicIssue,
    },
    context::Context,
    entities::{issue::Status, user::PublicUser},
    services::{FILES_SERVICE, PROTOCOL, RENDERER_SERVICE, USERS_SERVICE},
};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

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

/*
 * | Issues: {number} | severity1 | severity2 | severity3 |
 * ----------------------------------------------------------------
 * | Fixed            | {number}  | {number}  | {number}  |
 * | Not Fixed        | {number}  | {number}  | {number}  |
 *
 */
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Statistics {
    number_of_issues: usize,
    fixed_or_not: HashMap<String, [usize; 2]>,
}

impl Statistics {
    pub fn new(issues: &Vec<PublicIssue>) -> Self {
        let mut statistics = Statistics::default();

        for issue in issues {
            if issue.include {
                statistics.number_of_issues += 1;

                let fixed = (issue.status == Status::Fixed) as usize;

                match statistics.fixed_or_not.entry(issue.severity.clone()) {
                    Occupied(mut value) => value.get_mut()[fixed] += 1,
                    Vacant(place) => {
                        place.insert([1 - fixed, fixed]);
                    }
                }
            }
        }

        statistics
    }
}

fn generate_markdown_issue(issue: &PublicIssue) -> String {
    if !issue.include {
        return String::new();
    }

    //links
    let PublicIssue {
        name,
        status,
        category,
        description,
        feedback,
        severity,
        links,
        ..
    } = issue;

    let status = if status == &Status::Fixed {
        "Fixed"
    } else {
        "Not fixed"
    };

    let links: String = links.iter().map(|s| format!("- {s}\n")).collect();

    format!(
        "## {name}\n\n ### Severity: {severity}\n\n ### Status: {status}\n\n ### Category: {category}\n\n ### Links\n\n {links}\n\n {}\n\n ## Feedback\n\n {feedback}\n\n",
         description
    )
}

fn generate_markdown_statistics(statistics: &Statistics) -> String {
    [
        statistics.fixed_or_not.keys().fold(
            format!("| {} |", statistics.number_of_issues),
            |acc, sev| acc + &format!(" {} |", sev),
        ),
        statistics
            .fixed_or_not
            .values()
            .fold("| Fixed |".to_owned(), |acc, val| {
                acc + &format!(" {} |", val[1])
            }),
        statistics
            .fixed_or_not
            .values()
            .fold("| Not fixed |".to_owned(), |acc, val| {
                acc + &format!(" {} |", val[0])
            }),
    ]
    .concat()
}

fn generate_markdown_audit(audit: &PublicAudit) -> String {
    // make statistics
    let statistics = Statistics::new(&audit.issues);

    format!(
        "## Statistics\n\n {}\n\n ## Descritption\n\n {}\n\n",
        generate_markdown_statistics(&statistics),
        audit.description.clone(),
    )
}

fn generate_markdown(audit: &PublicAudit) -> String {
    audit
        .issues
        .iter()
        .fold(generate_markdown_audit(audit), |acc, issue| {
            acc + &generate_markdown_issue(issue)
        })
}

pub async fn create_report(context: Context, audit_id: String) -> anyhow::Result<PublicReport> {
    let audit = context
        .make_request::<PublicAudit>()
        .auth(context.auth().clone())
        .get(format!(
            "{}://{}/api/audit/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            audit_id
        ))
        .send()
        .await
        .unwrap()
        .json::<PublicAudit>()
        .await?;

    let markdown = generate_markdown(&audit);
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
            "http://{}/api/generate-report",
            RENDERER_SERVICE.as_str()
        ))
        .json(&input)
        .send()
        .await
        .unwrap()
        .bytes()
        .await?;

    let path = audit.id.clone() + ".pdf";

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

    let audit_change = AuditChange {
        report: Some(path.clone()),
        ..AuditChange::default()
    };

    let _ = context
        .make_request()
        .patch(format!(
            "{}://{}/api/audit/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            audit.id
        ))
        .auth(context.auth().clone())
        .json(&audit_change)
        .send()
        .await
        .unwrap();

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
