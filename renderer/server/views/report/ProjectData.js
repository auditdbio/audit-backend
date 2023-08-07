import React from 'react'
import ProjectLinksBlock from './blocks/ProjectLinksBlock.js'
import ReportBlocks from './ReportBlocks.js'

const ProjectData = ({ project }) => {
  return (
    <div className="project-data">
      <div className="table-of-contents" />

      <ReportBlocks blocks={project.report_data} />
    </div>
  )
}

export default ProjectData
