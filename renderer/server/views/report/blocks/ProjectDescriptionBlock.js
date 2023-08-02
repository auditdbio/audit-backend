import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'

const ProjectDescriptionBlock = ({ data }) => {
  return (
    <div className="report-block">
      <h2 className="report-block-title">{data.title}</h2>
      <div className="project-description page-break">
        <RenderMarkdown markdown={data.text} />
      </div>
    </div>
  )
}

export default ProjectDescriptionBlock
