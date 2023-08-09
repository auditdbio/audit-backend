import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'
import TitleLabel from '../TitleLabel.js'
import SeverityChip from './SeverityChip.js'

const IssueDataBlock = ({ data, num, subsectionLevel }) => {
  const getStatusColor = (status) => {
    if (status === 'Fixed') return '#09C010'
    if (status === 'NotFixed') return 'darkred'
    return 'black'
  }

  return (
    <div className="report-block">
      <h2 className={subsectionLevel ? 'subsection-title' : 'report-block-title'}>
        {num}. {data.title}
        <TitleLabel show={data.include_in_toc} />
      </h2>

      <div className="issue-data page-break">
        <div className="issue-info-wrapper">
          <span className="issue-info-title">Severity:</span>
          <SeverityChip severity={data.issue_data?.severity} />
        </div>
        {data.issue_data?.category && (
          <div className="issue-info-wrapper">
            <span className="issue-info-title">Category:</span>
            <span>{data.issue_data.category}</span>
          </div>
        )}
        <div className="issue-info-wrapper">
          <span className="issue-info-title">Status:</span>
          <span style={{ color: getStatusColor(data.issue_data?.status) }}>
            {data.issue_data?.status?.replace(/(?<=[a-z])([A-Z])/g, ' $1')}
          </span>
        </div>
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
