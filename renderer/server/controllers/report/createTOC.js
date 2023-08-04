import getPageForStrings from "../../utils/getPageForStrings.js"
import fs from "fs"
import fontkit from "@pdf-lib/fontkit"
import { rgb } from "pdf-lib"

const createTOC = async (project, pdfDoc, pdfBuffer, backgroundImage) => {
  const itemsForToc = project.report_data.reduce((acc, item) => {
    return item.include_in_toc ? [...acc, item.title] : acc
  }, [])
  const tableOfContents = await getPageForStrings(pdfBuffer, [...itemsForToc, 'Links'])

  const tocPage = await pdfDoc.insertPage(1)
  const { width, height } = tocPage.getSize()
  await tocPage.drawImage(backgroundImage, {
    x: 0,
    y: 0,
    width,
    height,
    blendMode: 'Normal',
  })

  const fontBytes = fs.readFileSync('server/assets/fonts/MartianMono-Regular.ttf')
  await pdfDoc.registerFontkit(fontkit)
  const tocFont = await pdfDoc.embedFont(fontBytes)
  await tocPage.drawText('Table of contents', { x: 40, y: 800, size: 20, color: rgb(0, 0, 0) })
  const tocFontSize = 10
  const lineHeight = 13
  const pageMaxWidth = 500
  const dotWidth = tocFont.widthOfTextAtSize('.', tocFontSize)
  let tocY = 770

  for (let i = 0; i < tableOfContents.length; i++) {
    const section = tableOfContents[i]
    const drawTextOptions = {
      x: 40,
      maxWidth: pageMaxWidth,
      font: tocFont,
      size: tocFontSize,
      lineHeight,
      color: rgb(0, 0, 0),
    }

    const lineWords = section.title.split(' ')
    let currentLine = ''
    for (let j = 0; j < lineWords.length; j++) {
      const currentLineWithWord = tocFont.widthOfTextAtSize(`${currentLine + lineWords[j]}. ${section.page}`, tocFontSize)
      if (currentLineWithWord <= 500) {
        currentLine += lineWords[j] + ' '
      } else {
        await tocPage.drawText(currentLine, { ...drawTextOptions, y: tocY })
        tocY -= lineHeight
        currentLine = `    ${lineWords[j]} `
      }
    }

    if (currentLine) {
      const lineWidth = tocFont.widthOfTextAtSize(`${currentLine}. ${section.page}`, tocFontSize)
      const numberOfDots = Math.floor((pageMaxWidth - lineWidth) / dotWidth)
      await tocPage.drawText(`${currentLine}.${'.'.repeat(numberOfDots)} ${section.page}`, { ...drawTextOptions, y: tocY })
      tocY -= lineHeight
    }
  }

  await pdfDoc.removePage(2)
}

export default createTOC
