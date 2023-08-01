import React from 'react'

const PlainTextBlock = ({ data }) => {
  return (
    <div className={data.text ? 'report-block' : ''}>
      <div className="report-block-title">{data.title}</div>
      <div className="report-plain-text">{data.text}</div>
    </div>
  )
}

export default PlainTextBlock
