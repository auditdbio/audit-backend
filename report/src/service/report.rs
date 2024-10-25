use actix_multipart::Multipart;
use futures::StreamExt;
use common::{
    api::{
        audits::{AuditChange, PublicAudit},
        issue::PublicIssue,
        report::{PublicReport, CreateReport},
        organization::{get_organization, GetOrganizationQuery},
    },
    auth::{Auth, Service},
    context::GeneralContext,
    entities::{
        audit::ReportType,
        issue::Status,
    },
    services::{API_PREFIX, FILES_SERVICE, FRONTEND, PROTOCOL, RENDERER_SERVICE, USERS_SERVICE},
};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyReportResponse {
    pub verified: bool,
    pub verification_code: Option<String>,
}

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
    pub audit_link: String,
    pub project_name: String,
    pub scope: Vec<String>,
    pub report_data: Vec<Section>,
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
    pub fn add_issue(mut self, issue: &PublicIssue, is_draft: bool) -> Self {
        let Some(section) = generate_issue_section(issue, is_draft) else {
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
        let sections = vec![
            ("Critical", &self.issues.critical),
            ("Major", &self.issues.major),
            ("Medium", &self.issues.medium),
            ("Minor", &self.issues.minor),
        ];

        sections
            .into_iter()
            .map(|(title, subsections)| {
                let mut section = Section {
                    typ: "plain_text".to_string(),
                    title: title.to_string(),
                    subsections: Some(subsections.clone()),
                    include_in_toc: true,
                    ..Default::default()
                };

                if subsections.is_empty() {
                    section.text = format!("No {} issues found.", title.to_lowercase());
                    section.subsections = None;
                }

                section
            })
            .collect()
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
                if issue.status == Status::Fixed {
                    statistics.total += 1;
                    match issue.severity.as_str() {
                        "Critical" => statistics.fixed.critical += 1,
                        "Major" => statistics.fixed.major += 1,
                        "Medium" => statistics.fixed.medium += 1,
                        "Minor" => statistics.fixed.minor += 1,
                        _ => {}
                    }
                } else if issue.status == Status::WillNotFix {
                    statistics.total += 1;
                    match issue.severity.as_str() {
                        "Critical" => statistics.not_fixed.critical += 1,
                        "Major" => statistics.not_fixed.major += 1,
                        "Medium" => statistics.not_fixed.medium += 1,
                        "Minor" => statistics.not_fixed.minor += 1,
                        _ => {}
                    }
                } else {
                    continue
                }
            }
        }

        statistics
    }
}

fn generate_issue_section(issue: &PublicIssue, is_draft: bool) -> Option<Section> {
    if !issue.include || (issue.status != Status::Fixed && issue.status != Status::WillNotFix) {
        return None;
    }

    if !is_draft && issue.status != Status::Fixed && issue.status != Status::WillNotFix {
        return None;
    }

    let PublicIssue {
        name,
        status,
        category,
        description,
        feedback,
        severity,
        ..
    } = issue;

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
            status: status.to_string(),
            category,
            links: issue.links.clone(),
        }),
        ..Default::default()
    })
}

fn generate_audit_sections(audit: &PublicAudit, issues: Vec<Section>) -> Vec<Section> {
    let statistics = Statistics::new(&audit.issues);

    let disclaimer = include_str!("../../templates/disclaimer.md").to_string();

    vec![
        Section {
            typ: "markdown".to_owned(),
            title: "Disclaimer".to_string(),
            text: disclaimer,
            include_in_toc: true,
            ..Default::default()
        },
        Section {
            typ: "plain_text".to_string(),
            title: "Summary".to_string(),
            include_in_toc: true,
            subsections: Some({
                let mut subsections = vec![
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
                ];
                if let Some(conclusion) = audit.conclusion.clone() {
                    subsections.push(Section {
                        typ: "markdown".to_string(),
                        title: "Conclusion".to_string(),
                        text: conclusion,
                        include_in_toc: true,
                        ..Default::default()
                    });
                }
                subsections
            }),
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

fn generate_data(audit: &PublicAudit, is_draft: bool) -> Vec<Section> {
    let issues = audit
        .issues
        .iter()
        .fold(IssueCollector::default(), |collector, i| {
            collector.add_issue(i, is_draft)
        })
        .into_issues();
    generate_audit_sections(audit, issues)
}

pub async fn create_report(
    context: GeneralContext,
    audit_id: String,
    data: CreateReport,
    code: Option<&String>
) -> anyhow::Result<PublicReport> {
    let auth = context.auth();

    let audit = context
        .make_request::<PublicAudit>()
        .auth(auth)
        .get(format!(
            "{}://{}/{}/audit/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            audit_id,
        ))
        .send()
        .await
        .unwrap()
        .json::<PublicAudit>()
        .await?;

    let mut auditor_name = audit.auditor_first_name.clone() + " " + &audit.auditor_last_name.clone();
    if audit.auditor_organization.is_some() {
        let organization_query = GetOrganizationQuery {
            with_members: Some(false),
        };
        let organization = get_organization(
            &context,
            audit.auditor_organization.clone().unwrap().id.parse()?,
            Some(organization_query),
        ).await.unwrap();
        auditor_name = organization.name;
    }

    let mut is_draft = data.is_draft.unwrap_or(false);
    if let Some(id) = auth.id() {
        if !audit.no_customer && audit.customer_id == id.to_hex() {
            is_draft = false;
        }
    }

    let report_data = generate_data(&audit, is_draft);
    let access_code = if let Some(code) = code {
        format!("?code={}", code)
    } else { "".to_string() };

    let input = RendererInput {
        auditor_name,
        profile_link: format!(
            "{}://{}/a/{}",
            PROTOCOL.as_str(),
            FRONTEND.as_str(),
            audit.auditor_id,
        ),
        audit_link: format!(
            "{}://{}/audit-info/{}{}",
            PROTOCOL.as_str(),
            FRONTEND.as_str(),
            audit.id,
            access_code,
        ),
        project_name: audit.project_name.clone(),
        scope: audit.scope,
        report_data,
    };

    let report = context
        .make_request()
        .post(format!(
            "{}://{}/{}/generate-report",
            PROTOCOL.as_str(),
            RENDERER_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .json(&input)
        .send()
        .await
        .unwrap()
        .bytes()
        .await?;

    let verification_code = if let Auth::Service(Service::Audits, _) = auth {
        let mut combined_bytes = Vec::new();
        combined_bytes.extend_from_slice(&report);
        Some(sha256::digest(&combined_bytes[..]))
    } else {
        None
    };

    let path = audit.id.clone() + ".pdf";

    let client = &context.client();
    let mut form = Form::new()
        .part("file", Part::bytes(report.to_vec()))
        .part("path", Part::text(path.clone()))
        .part("original_name", Part::text("report.pdf"))
        .part("private", Part::text("true"))
        .part("customerId", Part::text(audit.auditor_id))
        .part("auditorId", Part::text(audit.customer_id));

    if let Some(code) = code {
        form = form.part("access_code", Part::text(code.to_string()));
    }

    let _ = client
        .post(format!(
            "{}://{}/{}/file",
            PROTOCOL.as_str(),
            FILES_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .multipart(form)
        .send()
        .await?;

    if let Auth::Service(Service::Audits, _) = auth {
    } else {
        if !is_draft {
            let audit_change = AuditChange {
                report: Some(path.clone()),
                report_name: Some(format!("{} report.pdf", audit.project_name)),
                report_type: Some(ReportType::Generated),
                ..AuditChange::default()
            };

            let _ = context
                .make_request()
                .patch(format!(
                    "{}://{}/{}/audit/{}",
                    PROTOCOL.as_str(),
                    USERS_SERVICE.as_str(),
                    API_PREFIX.as_str(),
                    audit.id
                ))
                .auth(auth)
                .json(&audit_change)
                .send()
                .await
                .unwrap();
        }
    }

    Ok(PublicReport {
        path,
        verification_code,
    })
}

pub async fn verify_report(
    context: GeneralContext,
    audit_id: String,
    mut payload: Multipart,
) -> anyhow::Result<VerifyReportResponse> {
    let audit = context
        .make_request::<PublicAudit>()
        .auth(context.server_auth())
        .get(format!(
            "{}://{}/{}/audit/{}",
            PROTOCOL.as_str(),
            USERS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            audit_id,
        ))
        .send()
        .await
        .unwrap()
        .json::<PublicAudit>()
        .await?;

    let mut report = vec![];

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();

        match field.name() {
            "file" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    report.extend_from_slice(&data);
                }
            }
            _ => (),
        }
    }

    let verification_code = sha256::digest(&report[..]);
    let verified = Some(verification_code) == audit.verification_code;

    Ok(VerifyReportResponse {
        verified,
        verification_code: audit.verification_code,
    })
}
