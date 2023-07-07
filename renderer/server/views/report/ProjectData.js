import React from 'react'
import RenderMarkdown from './RenderMarkdown.js'

const ProjectData = ({ project }) => {
  return (
    <div className="project-data">
      {/*<div id="contents" className="section-title">*/}
      {/*  Table of contents:*/}
      {/*</div>*/}
      {/*<ul className="table-of-contents">*/}
      {/*  <li>*/}
      {/*    Summary*/}
      {/*    <ul className="table-of-contents">*/}
      {/*      <li>Project description</li>*/}
      {/*      <li>Scope</li>*/}
      {/*    </ul>*/}
      {/*  </li>*/}
      {/*  <li>Issue summary</li>*/}
      {/*  <li>Issues</li>*/}
      {/*</ul>*/}

      {/*<div id="description" className="section-title">*/}
      {/*  Description:*/}
      {/*</div>*/}

      <div className="project-description">
        <RenderMarkdown markdown={project.markdown} />
      </div>

      <div id="scope" className="section-title">
        Links:
      </div>

      <div className="project-scope">
        {project?.scope.map((link, idx) => (
          <a href={link} key={idx} className="project-link">
            {link}
          </a>
        ))}
      </div>
    </div>
  )
}

export default ProjectData
