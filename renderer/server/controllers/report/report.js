import puppeteer from 'puppeteer'
import { PDFDocument, rgb } from 'pdf-lib'
import fs from 'fs'
import getHTML from '../../views/html.js'
import createTOC from './createTOC.js'

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

export const generateReport = async (req, res) => {
  const project = req.body

  // --- Generate PDF from HTML page:
  const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] })
  const browserPage = await browser.newPage()
  await browserPage.setContent(getHTML(project), { waitUntil: 'networkidle0' })
  await browserPage.evaluateHandle('document.fonts.ready')
  await browserPage.evaluate(() => (document.body.style.zoom = 0.6))
  const pdfBuffer = await browserPage.pdf(pdfOptions)
  await browser.close()

  // --- Create table of contents:
  const pdfDoc = await PDFDocument.load(pdfBuffer)
  await createTOC(project, pdfDoc, pdfBuffer)

  // --- Adding a BG image and page number to PDF pages:
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
    await newPage.drawText(String(i + 1), {
      x: 570,
      y: 15,
      size: 10,
      color: rgb(0, 0, 0),
    })
    await newPage.drawPage(embeddedPage)
    await pdfDoc.removePage(i + 1)
  }

  // --- Create PDF document:
  const pdfBytesWithBackground = await pdfDoc.save()

  // --- Save generated report in temp directory:
  // fs.writeFileSync(`temp/output-${Date.now()}.pdf`, pdfBytesWithBackground)
  // fs.writeFileSync(`temp/output-${Date.now()}.html`, getHTML(project)) // save in html format for debugging

  res.contentType('application/pdf')
  res.end(pdfBytesWithBackground, 'binary')
}
