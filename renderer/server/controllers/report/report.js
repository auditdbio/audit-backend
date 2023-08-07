import puppeteer from 'puppeteer'
import { PDFDocument } from 'pdf-lib'
import fs from 'fs'
import getHTML from '../../views/html.js'
import { footerTemplate } from '../../views/footer.js'
import createTOC from './createTOC.js'

const pdfOptions = {
  format: 'A4',
  printBackground: true,
  displayHeaderFooter: true,
  headerTemplate: `<div/>`,
  footerTemplate,
  margin: {
    top: '1cm',
    bottom: '1cm',
    left: '1cm',
    right: '1cm',
  },
}

export const generateReport = async (req, res) => {
  const project = req.body

  // --- Generate PDF from HTML page:
  const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] })
  const browserPage = await browser.newPage()
  await browserPage.setContent(getHTML(project), { waitUntil: 'networkidle2' })
  await browserPage.evaluateHandle('document.fonts.ready')
  await browserPage.evaluate(() => (document.body.style.zoom = 0.5))
  const pdfBuffer = await browserPage.pdf(pdfOptions)
  await browser.close()

  // --- Adding a background image to a PDF pages:
  const pdfDoc = await PDFDocument.load(pdfBuffer)
  const backgroundFile = fs.readFileSync('server/assets/images/bg2.png')
  const coverImageFile = fs.readFileSync('server/assets/images/backgroundCover.png')
  const backgroundImage = await pdfDoc.embedPng(backgroundFile)
  const coverImage = await pdfDoc.embedPng(coverImageFile)
  const pdfDocPages = pdfDoc.getPages()

  const copiedPages = await pdfDoc.copyPages(pdfDoc, [...pdfDocPages.keys()])
  for (let i = 0; i < pdfDocPages.length; i++) {
    const { width, height } = pdfDocPages[i].getSize()
    const embeddedPage = await pdfDoc.embedPage(copiedPages[i])
    const newPage = await pdfDoc.insertPage(i)
    const background = i === 0 ? coverImage : backgroundImage
    await newPage.drawImage(background, {
      x: 0,
      y: 0,
      width,
      height,
      blendMode: 'Normal',
    })
    await newPage.drawPage(embeddedPage)
    await pdfDoc.removePage(i + 1)
  }

  // --- Create table of contents:
  await createTOC(project, pdfDoc, pdfBuffer, backgroundImage)

  // --- Create PDF file:
  const pdfBytesWithBackground = await pdfDoc.save()

  // --- Save generated report in temp directory:
  // fs.writeFileSync(`temp/output-${Date.now()}.pdf`, pdfBytesWithBackground)
  // fs.writeFileSync(`temp/output-${Date.now()}.html`, getHTML(project)) // save in html format for debugging

  res.contentType('application/pdf')
  res.end(pdfBytesWithBackground, 'binary')
}
