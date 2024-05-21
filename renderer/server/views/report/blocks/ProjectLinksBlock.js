import React from 'react'
import TitleLabel from '../TitleLabel.js'
import linkShortener from '../../../utils/linkShortener.js'

const ProjectLinksBlock = ({ data, num, subsectionLevel }) => {
  return (
    <div className="report-block">
      <h2 id="scope" className={subsectionLevel ? 'subsection-title' : 'report-block-title'}>
        {num}. {data?.title}
        <TitleLabel show={true} />
      </h2>
      <ul className="scope page-break project-links">
        {data?.links?.map((link, idx) => (
          <li key={idx} style={{marginBottom: '5px'}}>
            <a href={link} className="project-link">
              {linkShortener(link)}
            </a>
          </li>
        ))}
      </ul>
    </div>
  )
}

export default ProjectLinksBlock
