import React from 'react'
import { ISSUE_DATA, PLAIN_TEXT, PROJECT_DESCRIPTION, STATISTICS } from '../../constants/reportBlockTypes.js'
import ProjectDescriptionBlock from './blocks/ProjectDescriptionBlock.js'
import PlainTextBlock from './blocks/PlainTextBlock.js'
import IssueDataBlock from './blocks/IssueDataBlock.js'
import ProjectLinksBlock from './blocks/ProjectLinksBlock.js'
import ReportBlocks from "./ReportBlocks.js"

const ProjectData = ({ project }) => {
  return (
    <div className="project-data">
      <div className="table-of-contents" />

      <ReportBlocks blocks={project.report_data} />
      <ProjectLinksBlock project={project} />
    </div>
  )
}

export default ProjectData
