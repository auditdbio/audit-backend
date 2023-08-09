import fs from 'fs'
import fontkit from '@pdf-lib/fontkit'
import { rgb, StandardFonts } from 'pdf-lib'
import getPageForStrings from '../../utils/getPageForStrings.js'

const createTOC = async (project, pdfDoc, pdfBuffer) => {
  const tocReducer = (report_data, num) => {
    let idx = 1
    return report_data.reduce((acc, item) => {
      const numeration = num ? ` ${num}.${idx}` : idx
      let subsections = []
      if (item.subsections?.length) {
        subsections = tocReducer(item.subsections, numeration)
      }
      const title = `${numeration}. ${item.title}`
      if (item.include_in_toc) {
        idx += 1
        return [...acc, title, ...subsections]
      }
      return acc
    }, [])
  }

  const itemsForToc = tocReducer(project.report_data)
  const tableOfContents = await getPageForStrings(pdfBuffer, itemsForToc)

  let tocPagesCounter = 1
  let tocPage = await pdfDoc.insertPage(tocPagesCounter)

  const fontBytes = fs.readFileSync('server/assets/fonts/RobotoMono-Regular.ttf')
  await pdfDoc.registerFontkit(fontkit)
  const tocFont = await pdfDoc.embedFont(fontBytes)
  await tocPage.drawText('Table of contents', { x: 40, y: 800, size: 20, color: rgb(0, 0, 0) })
  const tocFontSize = 10
  const lineHeight = 13
  const pageMaxWidth = 500
  const dotWidth = tocFont.widthOfTextAtSize('.', tocFontSize)
  const countOfLines = tableOfContents.reduce((acc, item) => {
    return acc + Math.ceil(tocFont.widthOfTextAtSize(item.title + 5, tocFontSize) / 500)
  }, 0)
  const countOfTocPages = Math.ceil(countOfLines / 55)

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
      const currentLineWithWord = tocFont.widthOfTextAtSize(
        `${currentLine + lineWords[j]}. ${section.page + countOfTocPages}`,
        tocFontSize
      )
      if (currentLineWithWord <= 500) {
        currentLine += lineWords[j] + ' '
      } else {
        await tocPage.drawText(currentLine, { ...drawTextOptions, y: tocY })
        tocY -= lineHeight
        currentLine = `  ${lineWords[j]} `
      }
    }

    if (currentLine) {
      const lineWidth = tocFont.widthOfTextAtSize(`${currentLine}. ${section.page + countOfTocPages}`, tocFontSize)
      let numberOfDots = Math.max(Math.floor((pageMaxWidth - lineWidth) / dotWidth), 0)
      await tocPage.drawText(`${currentLine}.${'.'.repeat(numberOfDots)} ${section.page + countOfTocPages}`, {
        ...drawTextOptions,
        y: tocY,
      })
      tocY -= lineHeight
    }

    if (tocY <= 65) {
      tocPagesCounter++
      tocPage = await pdfDoc.insertPage(tocPagesCounter)
      tocY = 800
    }
  }
}

export default createTOC
