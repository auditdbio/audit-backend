use actix_multipart::Multipart;
use futures::StreamExt;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

use common::{
    api::{
        audits::{AuditChange, PublicAudit},
        issue::PublicIssue,
        file::PublicMetadata,
        report::{PublicReport, CreateReport},
        organization::{get_organization, GetOrganizationQuery},
    },
    auth::{Auth, Service},
    context::GeneralContext,
    entities::{
        audit::{PublicAuditStatus, ReportType},
        file::{FileEntity, ParentEntitySource},
        issue::Status,
    },
    error::{self, AddCode},
    services::{API_PREFIX, AUDITS_SERVICE, FILES_SERVICE, FRONTEND, PROTOCOL, RENDERER_SERVICE},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifyReportResponse {
    pub verified: bool,
    pub report_sha: Option<String>,
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
    if !issue.include || (issue.status == Status::Draft && !is_draft) {
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
                ];

                if !audit.scope.is_empty() {
                    subsections.push(Section {
                        typ: "scope".to_string(),
                        title: "Scope".to_string(),
                        links: Some(audit.scope.clone()),
                        include_in_toc: true,
                        ..Default::default()
                    });
                }

                if let Some(conclusion) = audit.conclusion.clone() {
                    if !conclusion.trim().is_empty() {
                        subsections.push(Section {
                            typ: "markdown".to_string(),
                            title: "Conclusion".to_string(),
                            text: conclusion,
                            include_in_toc: true,
                            ..Default::default()
                        });
                    }
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
) -> error::Result<PublicReport> {
    let auth = context.auth();

    let audit = context
        .make_request::<PublicAudit>()
        .auth(auth)
        .get(format!(
            "{}://{}/{}/audit/{}",
            PROTOCOL.as_str(),
            AUDITS_SERVICE.as_str(),
            API_PREFIX.as_str(),
            audit_id,
        ))
        .send()
        .await
        .unwrap()
        .json::<PublicAudit>()
        .await?;

    if !audit.no_customer && audit.status == PublicAuditStatus::Resolved && audit.report.is_some() {
        return Err(anyhow::anyhow!("Cannot generate report for resolved audit").code(400));
    }

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
            "{}://{}/audit/{}{}",
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

    let report_sha = if let Auth::Service(Service::Audits, _) = auth {
        let mut combined_bytes = Vec::new();
        combined_bytes.extend_from_slice(&report);
        Some(sha256::digest(&combined_bytes[..]))
    } else {
        None
    };

    let file_entity = if is_draft {
        FileEntity::Temporary
    } else {
        FileEntity::Report
    };

    let original_name = format!("{} report.pdf", audit.project_name);

    let client = &context.client();
    let mut form = Form::new()
        .part("file", Part::bytes(report.to_vec()))
        .part("original_name", Part::text(original_name))
        .part("file_entity", Part::text(file_entity.to_string()))
        .part("parent_entity_id", Part::text(audit.id.clone()))
        .part("parent_entity_source", Part::text(ParentEntitySource::Audit.to_string()))
        .part("private", Part::text("true"))
        .part("customerId", Part::text(audit.auditor_id))
        .part("auditorId", Part::text(audit.customer_id));

    if let Some(code) = code {
        form = form.part("access_code", Part::text(code.to_string()));
    }

    let file_meta_str = client
        .post(format!(
            "{}://{}/{}/file",
            PROTOCOL.as_str(),
            FILES_SERVICE.as_str(),
            API_PREFIX.as_str(),
        ))
        .bearer_auth(auth.to_token().unwrap())
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    let file_meta: PublicMetadata = serde_json::from_str(&file_meta_str)?;
    let report_name = format!(
        "{}.{}",
        file_meta.original_name.unwrap_or("Report".to_string()),
        file_meta.extension,
    );

    if let Auth::Service(Service::Audits, _) = auth {
    } else {
        if !is_draft && audit.status != PublicAuditStatus::Resolved {
            let audit_change = AuditChange {
                report: Some(file_meta.id.clone()),
                report_type: Some(ReportType::Generated),
                ..AuditChange::default()
            };

            let _ = context
                .make_request()
                .patch(format!(
                    "{}://{}/{}/audit/{}",
                    PROTOCOL.as_str(),
                    AUDITS_SERVICE.as_str(),
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
        file_id: file_meta.id,
        report_name,
        is_draft,
        report_sha,
    })
}

pub async fn verify_report(
    context: GeneralContext,
    audit_id: String,
    mut payload: Multipart,
) -> error::Result<VerifyReportResponse> {
    let audit = context
        .make_request::<PublicAudit>()
        .auth(context.server_auth())
        .get(format!(
            "{}://{}/{}/audit/{}",
            PROTOCOL.as_str(),
            AUDITS_SERVICE.as_str(),
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

    if report.is_empty() {
        return Err(anyhow::anyhow!("'file' field is required").code(400));
    }

    if audit.report_sha.is_none() {
        return Err(anyhow::anyhow!("There is no verification code for this audit.").code(204));
    }

    let report_sha = sha256::digest(&report[..]);
    let verified = Some(report_sha) == audit.report_sha;

    Ok(VerifyReportResponse {
        verified,
        report_sha: audit.report_sha,
    })
}
