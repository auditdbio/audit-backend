import React from 'react'
import ReportBlocks from './ReportBlocks.js'

const ProjectData = ({ project }) => {
  return (
    <div className="project-data">
      <ReportBlocks blocks={project.report_data} />
    </div>
  )
}

export default ProjectData
