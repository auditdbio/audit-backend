import React from 'react'
import { ISSUE_DATA, PLAIN_TEXT, PROJECT_DESCRIPTION, STATISTICS } from '../../constants/reportBlockTypes.js'
import ProjectDescriptionBlock from './blocks/ProjectDescriptionBlock.js'
import PlainTextBlock from './blocks/PlainTextBlock.js'
import IssueDataBlock from './blocks/IssueDataBlock.js'

const ReportBlocks = ({ blocks, num }) => {
  return (
    <>
      {blocks?.map((reportBlock, idx) => {
        const numeration = num ? `${num}.${idx + 1}` : idx + 1

        if (reportBlock.type === PROJECT_DESCRIPTION || reportBlock.type === STATISTICS) {
          return (
            <>
              <ProjectDescriptionBlock data={reportBlock} num={numeration} />
              {reportBlock.subsections?.length && <ReportBlocks blocks={reportBlock.subsections} num={numeration} />}
            </>
          )
        } else if (reportBlock.type === PLAIN_TEXT) {
          return (
            <>
              <PlainTextBlock data={reportBlock} num={numeration} />
              {reportBlock.subsections?.length && <ReportBlocks blocks={reportBlock.subsections} num={numeration} />}
            </>
          )
        } else if (reportBlock.type === ISSUE_DATA) {
          return (
            <>
              <IssueDataBlock data={reportBlock} num={numeration} />
              {reportBlock.subsections?.length && <ReportBlocks blocks={reportBlock.subsections} num={numeration} />}
            </>
          )
        }
      })}
    </>
  )
}

export default ReportBlocks
