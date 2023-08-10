import { PDFName, PDFString } from 'pdf-lib'

const createPageLinkAnnotation = async (pdfDoc, tableOfContents, profileLink) => {
  const pdfDocPages = pdfDoc.getPages()

  const createProfileLinkAnnot = (profileLink) => {
    return pdfDoc.context.register(
      pdfDoc.context.obj({
        Type: 'Annot',
        Subtype: 'Link',
        Rect: [60, 360, 150, 390],
        Border: [0, 0, 0],
        C: [1, 1, 1],
        A: { Type: 'Action', S: 'URI', URI: PDFString.of(profileLink) },
      }))
  }

  pdfDocPages[0].node.set(PDFName.of('Annots'), pdfDoc.context.obj([createProfileLinkAnnot(profileLink)]))

  const createAnnot = (pageRef, tocStringCoordY, destCoordY, tocStringNumberOfLines) => {
    return pdfDoc.context.register(
      pdfDoc.context.obj({
        Type: 'Annot',
        Subtype: 'Link',
        Rect: [
          40, // lower left x coord
          tocStringCoordY - 2 - (13 * tocStringNumberOfLines), // lower left y coord
          540, // upper right x coord
          tocStringCoordY + 8, // upper right y coord
        ],
        Border: [0, 0, 0], // Border for the link
        C: [1, 1, 1], // Make the border color white: rgb(1, 1, 1)
        Dest: [pageRef, 'XYZ', null, destCoordY + 20, null], // Page to be visited when the link is clicked
      }))
  }

  let links = []
  let currentTocPage = tableOfContents[0].tocPage

  for (let i = 0; i < tableOfContents.length; i++) {
    const destPage = tableOfContents[i].destPage + 1
    const pageRef = pdfDocPages[destPage].ref
    const destCoordY = tableOfContents[i].destCoordY || null
    const tocStringCoordY = tableOfContents[i].tocStringCoordY
    const tocStringNumberOfLines = tableOfContents[i].tocStringNumberOfLines
    const link = createAnnot(pageRef, tocStringCoordY, destCoordY, tocStringNumberOfLines)

    if (tableOfContents[i].tocPage !== currentTocPage) {
      pdfDocPages[currentTocPage].node.set(PDFName.of('Annots'), pdfDoc.context.obj(links))
      currentTocPage = tableOfContents[i].tocPage
      links = []
    }

    links.push(link)
  }

  pdfDocPages[currentTocPage].node.set(PDFName.of('Annots'), pdfDoc.context.obj(links))
}

export default createPageLinkAnnotation