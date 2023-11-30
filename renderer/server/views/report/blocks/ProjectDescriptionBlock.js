import React from 'react'
import RenderMarkdown from '../RenderMarkdown.js'
import TitleLabel from '../TitleLabel.js'

const ProjectDescriptionBlock = ({ data, num, subsectionLevel }) => {
  return (
    <div className="report-block">
      <h2 className={subsectionLevel ? 'subsection-title' : 'report-block-title'}>
        {num}. {data?.title}
        <TitleLabel show={data?.include_in_toc} />
      </h2>
      <div className="project-description page-break">
        <RenderMarkdown markdown={data?.text} />
      </div>
    </div>
  )
}

export default ProjectDescriptionBlock
