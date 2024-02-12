import style from '../assets/index.js'
import reactRender from '../utils/reactRender.js'

const getHTML = (project) => {
  return `
    <html lang="en">
      <head>
        <title>Report</title>
        <meta charset="utf-8" />
        ${style}
      </head>
      <body>
        <div id="root">
          ${reactRender(project)}
        </div>
      </body>
    </html>
  `
}

export default getHTML
