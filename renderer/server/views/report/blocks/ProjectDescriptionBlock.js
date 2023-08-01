import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'

const ProjectDescriptionBlock = ({ data }) => {
  return (
    <div className="report-block">
      <div className="report-block-title">{data.title}</div>
      <div className="project-description">
        <RenderMarkdown markdown={data.text} />
      </div>
    </div>
  )
}

export default ProjectDescriptionBlock
