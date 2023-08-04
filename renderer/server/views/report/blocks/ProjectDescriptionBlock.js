import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'
import TitleLabel from '../TitleLabel.js'

const ProjectDescriptionBlock = ({ data }) => {
  return (
    <div className="report-block">
      <h2 className="report-block-title"> 
        {data.title}
        <TitleLabel show={data.include_in_toc} />
      </h2>
      <div className="project-description page-break">
        <RenderMarkdown markdown={data.text} />
      </div>
    </div>
  )
}

export default ProjectDescriptionBlock
