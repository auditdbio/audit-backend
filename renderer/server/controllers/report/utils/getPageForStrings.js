import pdfjsLib from 'pdfjs-dist/legacy/build/pdf.js'

async function getPageForStrings(pdfBuffer, titlesArray) {
  const tableOfContents = []
  let foundHeadings = []

  const pdfData = new Uint8Array(pdfBuffer)

  const loadingTask = pdfjsLib.getDocument(pdfData)
  await loadingTask.promise
    .then((doc) => {
      const numPages = doc.numPages
      let lastPromise // will be used to chain promises
      lastPromise = doc.getMetadata()

      const loadPage = (pageNum) => {
        return doc.getPage(pageNum).then((page) => {
          return page.getTextContent().then((content) => {
            const pageHeadingsYCoords = []
            const strings = content.items.map((item) => {
              if (item.str === '|||') {
                pageHeadingsYCoords.push(item.transform[5])
              }
              return item.str
            })
            const text = strings.join('').replace(/ /g, '')

            let headingCoordsIdx = 0
            titlesArray.forEach((title) => {
              if (foundHeadings.includes(title)) return
              if (text.includes(title.replace(/^ *(\d\.)* /g, '').replace(/ /g, '') + '|||')) {
                if (!pageHeadingsYCoords[headingCoordsIdx]) return
                foundHeadings.push(title)
                tableOfContents.push({
                  title,
                  page: pageNum,
                  coordY: pageHeadingsYCoords[headingCoordsIdx],
                })
                headingCoordsIdx++
              }
            })
            page.cleanup() // Release page resources.
          })
        })
      }
      for (let i = 1; i <= numPages; i++) {
        lastPromise = lastPromise.then(loadPage.bind(null, i))
      }
      return lastPromise
    })
    .catch((err) => console.error('Error: ' + err))

  return tableOfContents
}

export default getPageForStrings
