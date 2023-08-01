import React from 'react'
import { ISSUE_DATA, PLAIN_TEXT, PROJECT_DESCRIPTION, STATISTICS } from '../../constants/reportBlockTypes.js'
import ProjectDescriptionBlock from './blocks/ProjectDescriptionBlock.js'
import PlainTextBlock from './blocks/PlainTextBlock.js'
import IssueDataBlock from './blocks/IssueDataBlock.js'

const ProjectData = ({ project }) => {
  return (
    <div className="project-data">
      {project.report_data?.map((reportBlock) => {
        if (reportBlock.type === PROJECT_DESCRIPTION || reportBlock.type === STATISTICS) {
          return <ProjectDescriptionBlock data={reportBlock} />
        } else if (reportBlock.type === PLAIN_TEXT) {
          return <PlainTextBlock data={reportBlock} />
        } else if (reportBlock.type === ISSUE_DATA) {
          return <IssueDataBlock data={reportBlock} />
        }
      })}

      <div className="report-block">
        <div id="scope" className="report-block-title">
          Links:
        </div>
        <div className="project-scope">
          {project?.scope.map((link, idx) => (
            <a href={link} key={idx} className="project-link">
              {link}
            </a>
          ))}
        </div>
      </div>
    </div>
  )
}

export default ProjectData
