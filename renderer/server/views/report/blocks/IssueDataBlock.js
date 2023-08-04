import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'
import TitleLabel from '../TitleLabel.js'

const IssueDataBlock = ({ data, num }) => {
  return (
    <div className="report-block issue-block">
      <h2 className="report-issue-title">
        {num}. {data.title}
        <TitleLabel show={data.include_in_toc} />
      </h2>

      <div className="issue-data page-break">
        <div>
          <b>Status:</b> {data.issue_data?.status}
        </div>
        <div>
          <b>Severity:</b> {data.issue_data?.severity}
        </div>
        {data.issue_data?.category && (
          <div>
            <b>Category:</b> {data.issue_data.category}
          </div>
        )}
      </div>

      <div className="project-description">
        <RenderMarkdown markdown={data.text} />
      </div>

      {!!data.issue_data?.links?.length && (
        <div className="scope issue-links">
          {data.issue_data.links.map((link, idx) => (
            <a href={link} key={idx} className="issue-link">
              {link}
            </a>
          ))}
        </div>
      )}

      {data.feedback && (
        <div className="issue-feedback">
          <div className="issue-feedback-title">Feedback:</div>
          <RenderMarkdown markdown={data.feedback} />
        </div>
      )}
    </div>
  )
}

export default IssueDataBlock
