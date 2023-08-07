import React from 'react'
import TitleLabel from '../TitleLabel.js'

const ProjectLinksBlock = ({ data, num }) => {
  return (
    <div className="report-block">
      <h2 id="scope" className="report-block-title">
        {num}. {data.title}
        <TitleLabel show={true} />
      </h2>
      <div className="scope page-break">
        {data.links?.map((link, idx) => (
          <a href={link} key={idx} className="project-link">
            {link}
          </a>
        ))}
      </div>
    </div>
  )
}

export default ProjectLinksBlock
