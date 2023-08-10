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
    services::{FILES_SERVICE, FRONTEND, PROTOCOL, RENDERER_SERVICE, USERS_SERVICE},
};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueData {
    pub severity: Option<String>,
    pub status: String,
    pub category: Option<String>,
    pub links: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Section {
    #[serde(rename = "type")]
    pub typ: String,
    pub title: String,
    pub text: String,
    pub include_in_toc: bool,
    pub feedback: Option<String>,
    pub issue_data: Option<IssueData>,
    pub subsections: Option<Vec<Section>>,
    pub statistics: Option<Statistics>,
    pub links: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RendererInput {
    pub auditor_name: String,
    pub profile_link: String,
    pub project_name: String,
    pub scope: Vec<String>,
    pub report_data: Vec<Section>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicReport {
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssuesCount<T> {
    critical: T,
    major: T,
    medium: T,
    minor: T,
}

#[derive(Debug, Clone, Default)]
pub struct IssueCollector {
    issues: IssuesCount<Vec<Section>>,
}

impl IssueCollector {
    pub fn add_issue(mut self, issue: &PublicIssue) -> Self {
        let Some(section) = generate_issue_section(issue) else {
            return self;
        };

        match issue.severity.as_str() {
            "Critical" => self.issues.critical.push(section),
            "Major" => self.issues.major.push(section),
            "Medium" => self.issues.medium.push(section),
            "Minor" => self.issues.minor.push(section),
            _ => {}
        };

        self
    }
    pub fn into_issues(self) -> Vec<Section> {
        vec![
            Section {
                typ: "plain_text".to_string(),
                title: "Critical".to_string(),
                subsections: Some(self.issues.critical),
                include_in_toc: true,
                ..Default::default()
            },
            Section {
                typ: "plain_text".to_string(),
                title: "Major".to_string(),
                subsections: Some(self.issues.major),
                include_in_toc: true,
                ..Default::default()
            },
            Section {
                typ: "plain_text".to_string(),
                title: "Medium".to_string(),
                subsections: Some(self.issues.medium),
                include_in_toc: true,
                ..Default::default()
            },
            Section {
                typ: "plain_text".to_string(),
                title: "Minor".to_string(),
                subsections: Some(self.issues.minor),
                include_in_toc: true,
                ..Default::default()
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Statistics {
    total: usize,
    fixed: IssuesCount<usize>,
    not_fixed: IssuesCount<usize>,
}

impl Statistics {
    pub fn new(issues: &Vec<PublicIssue>) -> Self {
        let mut statistics = Statistics::default();

        for issue in issues {
            if issue.include {
                statistics.total += 1;

                if issue.status == Status::Fixed {
                    match issue.severity.as_str() {
                        "Critical" => statistics.fixed.critical += 1,
                        "Major" => statistics.fixed.major += 1,
                        "Medium" => statistics.fixed.medium += 1,
                        "Minor" => statistics.fixed.minor += 1,
                        _ => {}
                    }
                } else {
                    match issue.severity.as_str() {
                        "Critical" => statistics.not_fixed.critical += 1,
                        "Major" => statistics.not_fixed.major += 1,
                        "Medium" => statistics.not_fixed.medium += 1,
                        "Minor" => statistics.not_fixed.minor += 1,
                        _ => {}
                    }
                }
            }
        }

        statistics
    }
}

fn generate_issue_section(issue: &PublicIssue) -> Option<Section> {
    if !issue.include {
        return None;
    }

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
        "NotFixed"
    }
    .to_string();

    let feedback = if !feedback.is_empty() {
        Some(feedback.clone())
    } else {
        None
    };

    let severity = if !severity.is_empty() {
        Some(severity.clone())
    } else {
        None
    };

    let category = if !category.is_empty() {
        Some(category.clone())
    } else {
        None
    };

    Some(Section {
        typ: "issue_data".to_string(),
        title: name.clone(),
        text: description.clone(),
        include_in_toc: true,
        feedback,
        issue_data: Some(IssueData {
            severity,
            status,
            category,
            links: issue.links.clone(),
        }),
        ..Default::default()
    })
}

fn generate_audit_sections(audit: &PublicAudit, issues: Vec<Section>) -> Vec<Section> {
    let statistics = Statistics::new(&audit.issues);

    /*
     * Table of contests
     * Disclamer
     * Summary
     *     Project description
     *     Scope
     */
    let disclamer = include_str!("../../templates/disclamer.txt").to_string();

    vec![
        Section {
            typ: "plain_text".to_owned(),
            title: "Disclamer".to_string(),
            text: disclamer,
            include_in_toc: true,
            ..Default::default()
        },
        Section {
            typ: "plain_text".to_string(),
            title: "Summary".to_string(),
            include_in_toc: true,
            subsections: Some(vec![
                Section {
                    typ: "project_description".to_string(),
                    title: "Project Description".to_string(),
                    text: audit.description.clone(),
                    include_in_toc: true,
                    ..Default::default()
                },
                Section {
                    typ: "scope".to_string(),
                    title: "Scope".to_string(),
                    links: Some(audit.scope.clone()),
                    include_in_toc: true,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
        Section {
            typ: "statistics".to_string(),
            title: "Issue Summary".to_string(),
            statistics: Some(statistics),
            include_in_toc: true,
            ..Default::default()
        },
        Section {
            typ: "plain_text".to_string(),
            title: "Issues".to_string(),
            subsections: Some(issues),
            include_in_toc: true,
            ..Default::default()
        },
    ]
}

fn generate_data(audit: &PublicAudit) -> Vec<Section> {
    let issues = audit
        .issues
        .iter()
        .fold(IssueCollector::default(), |collector, i| {
            collector.add_issue(i)
        })
        .into_issues();
    generate_audit_sections(audit, issues)
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

    let report_data = generate_data(&audit);
    let input = RendererInput {
        auditor_name: audit.auditor_first_name + " " + &audit.auditor_last_name,
        profile_link: format!(
            "{}://{}/user/{}/auditor",
            PROTOCOL.as_str(),
            FRONTEND.as_str(),
            audit.auditor_id
        ),
        project_name: audit.project_name,
        scope: audit.scope,
        report_data,
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
