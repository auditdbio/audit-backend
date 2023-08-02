import React from 'react'

const PlainTextBlock = ({ data }) => {
  return (
    <div className={data.text ? 'report-block' : ''}>
      <h2 className="report-block-title">{data.title}</h2>
      {data.text && <div className="report-plain-text page-break">{data.text}</div>}
    </div>
  )
}

export default PlainTextBlock
