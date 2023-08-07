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
            const strings = content.items.map((item) => item.str)
            const text = strings.join('').replace(/ /g, '')
            titlesArray.forEach((title) => {
              if (foundHeadings.includes(title)) return
              if (text.includes(title.replace(/^ *(\d\.)* /g, '').replace(/ /g, '') + '|||')) {
                foundHeadings.push(title)
                tableOfContents.push({
                  title,
                  page: pageNum,
                })
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
