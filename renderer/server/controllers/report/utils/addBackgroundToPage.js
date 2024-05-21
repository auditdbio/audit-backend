import fs from 'fs'
import { rgb } from 'pdf-lib'

const addBackgroundToPages = async (pdfDoc) => {
  // const backgroundFile = fs.readFileSync('server/assets/images/bg2.png')
  // const coverImageFile = fs.readFileSync('server/assets/images/backgroundCover.png')
  // const backgroundImage = await pdfDoc.embedPng(backgroundFile)
  // const coverImage = await pdfDoc.embedPng(coverImageFile)

  const pdfDocPages = pdfDoc.getPages()
  // const copiedPages = await pdfDoc.copyPages(pdfDoc, [...pdfDocPages.keys()])

  for (let i = 0; i < pdfDocPages.length; i++) {
    // const { width, height } = pdfDocPages[i].getSize()
    // await copiedPages[i].drawText(' ')
    // const embeddedPage = await pdfDoc.embedPage(copiedPages[i])
    // const newPage = await pdfDoc.insertPage(i)
    // const background = i === 0 ? coverImage : backgroundImage
    // await newPage.drawImage(background, {
    //   x: 0,
    //   y: 0,
    //   width,
    //   height,
    //   blendMode: 'Normal',
    // })

    // Add number of page:
    await pdfDocPages[i].drawText(String(i + 1), {
      x: 570,
      y: 15,
      size: 10,
      color: rgb(0, 0, 0),
    })
    // await newPage.drawPage(embeddedPage)
    // await pdfDoc.removePage(i + 1)
  }
}

export default addBackgroundToPages
