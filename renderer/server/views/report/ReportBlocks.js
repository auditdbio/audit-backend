import React from 'react'
import {
  ISSUE_DATA,
  MARKDOWN,
  PLAIN_TEXT,
  PROJECT_DESCRIPTION,
  SCOPE,
  STATISTICS,
} from '../../constants/reportBlockTypes.js'
import ProjectDescriptionBlock from './blocks/ProjectDescriptionBlock.js'
import PlainTextBlock from './blocks/PlainTextBlock.js'
import IssueDataBlock from './blocks/IssueDataBlock.js'
import ProjectLinksBlock from './blocks/ProjectLinksBlock.js'
import StatisticsBlock from './blocks/StatisticsBlock.js'

const ReportBlocks = ({ blocks, num, subsectionLevel = 0 }) => {
  const paddingLeft = `${subsectionLevel * 20}px`

  return (
    <div style={{ paddingLeft }}>
      {blocks?.map((reportBlock, idx) => {
        const numeration = num ? `${num}.${idx + 1}` : idx + 1

        if (reportBlock.type === PROJECT_DESCRIPTION || reportBlock.type === MARKDOWN) {
          return (
            <>
              <ProjectDescriptionBlock data={reportBlock} num={numeration} subsectionLevel={subsectionLevel} />
              {reportBlock.subsections?.length && (
                <ReportBlocks blocks={reportBlock.subsections} num={numeration} subsectionLevel={subsectionLevel + 1} />
              )}
            </>
          )
        } else if (reportBlock.type === PLAIN_TEXT) {
          return (
            <>
              <PlainTextBlock data={reportBlock} num={numeration} subsectionLevel={subsectionLevel} />
              {reportBlock.subsections?.length && (
                <ReportBlocks blocks={reportBlock.subsections} num={numeration} subsectionLevel={subsectionLevel + 1} />
              )}
            </>
          )
        } else if (reportBlock.type === ISSUE_DATA) {
          return (
            <>
              <IssueDataBlock data={reportBlock} num={numeration} subsectionLevel={subsectionLevel} />
              {reportBlock.subsections?.length && (
                <ReportBlocks blocks={reportBlock.subsections} num={numeration} subsectionLevel={subsectionLevel + 1} />
              )}
            </>
          )
        } else if (reportBlock.type === SCOPE) {
          return <ProjectLinksBlock data={reportBlock} num={numeration} subsectionLevel={subsectionLevel} />
        } else if (reportBlock.type === STATISTICS) {
          return <StatisticsBlock data={reportBlock} num={numeration} subsectionLevel={subsectionLevel} />
        }
      })}
    </div>
  )
}

export default ReportBlocks
