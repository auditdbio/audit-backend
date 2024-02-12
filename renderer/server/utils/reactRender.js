import React from 'react'
import ReactDOMServer from 'react-dom/server'
import TitlePage from '../views/report/TitlePage.js'
import ProjectData from '../views/report/ProjectData.js'

const reactRender = (project) => {
  return ReactDOMServer.renderToString(
    <>
      <TitlePage project={project} />
      <ProjectData project={project} />
    </>
  )
}

export default reactRender
