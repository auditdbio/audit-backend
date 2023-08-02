import React from 'react'

const ProjectLinksBlock = ({ project }) => {
  return (
    <div className="report-block">
      <h2 id="scope" className="report-block-title">
        Links
      </h2>
      <div className="scope page-break">
        {project?.scope.map((link, idx) => (
          <a href={link} key={idx} className="project-link">
            {link}
          </a>
        ))}
      </div>
    </div>
  )
}

export default ProjectLinksBlock
