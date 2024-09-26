import puppeteer from 'puppeteer'
import { PDFDocument } from 'pdf-lib'
import fs from 'fs'
import getHTML from '../../views/html.js'
import createTOC from './utils/createTOC.js'
import createPageLinkAnnotation from './utils/createPageLinkAnnotation.js'
import addBackgroundToPages from './utils/addBackgroundToPage.js'
import { FRONTEND, PROTOCOL } from "../../constants/reportLink.js"

const pdfOptions = {
  format: 'A4',
  printBackground: true,
  displayHeaderFooter: false,
  margin: {
    top: '1cm',
    bottom: '1cm',
    left: '1cm',
    right: '1cm',
  },
}

export const generateReport = async (req, res, next) => {
  try {
    const project = req.body

    const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] })
    const browserPage = await browser.newPage()
    await browserPage.setContent(getHTML(project), { waitUntil: 'networkidle0' })
    await browserPage.evaluateHandle('document.fonts.ready')
    await browserPage.evaluate(() => (document.body.style.zoom = 0.6))
    const pdfBuffer = await browserPage.pdf(pdfOptions)
    await browser.close()

    const pdfDoc = await PDFDocument.load(pdfBuffer)

    const { tableOfContentsWithCoords, tocPagesCounter } = await createTOC(project, pdfDoc, pdfBuffer)
    await addBackgroundToPages(pdfDoc)
    await createPageLinkAnnotation(
      pdfDoc,
      tableOfContentsWithCoords,
      tocPagesCounter,
      project?.profile_link || `${PROTOCOL}://${FRONTEND}/disclaimer/`,
      project?.audit_link || `${PROTOCOL}://${FRONTEND}/disclaimer/`,
    )

    const pdfBytes = await pdfDoc.save()

    // --- Save generated report in temp directory:
    // fs.writeFileSync(`temp/output-${Date.now()}.pdf`, pdfBytes)
    // fs.writeFileSync(`temp/output-${Date.now()}.html`, getHTML(project)) // save in html format for debugging

    res.contentType('application/pdf')
    res.end(pdfBytes, 'binary')
  } catch (err) {
    next(err)
  }
}
