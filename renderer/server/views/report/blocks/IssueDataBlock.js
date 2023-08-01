import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'

const IssueDataBlock = ({ data }) => {
  return (
    <div className="report-block issue-block">
      <div className="report-issue-title">{data.title}</div>

      <div className="issue-data">
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
