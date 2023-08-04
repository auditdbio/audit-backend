import React from 'react'
import TitleLabel from '../TitleLabel.js'

const PlainTextBlock = ({ data, num }) => {
  return (
    <div className={data.text ? 'report-block' : ''}>
      <h2 className="report-block-title">
        {num}. {data.title}
        <TitleLabel show={data.include_in_toc} />
      </h2>
      {data.text && <div className="report-plain-text page-break">{data.text}</div>}
    </div>
  )
}

export default PlainTextBlock
