import React from 'react'
import TitleLabel from '../TitleLabel.js'
import SeverityChip from './SeverityChip.js'

const StatisticsBlock = ({ data, num, subsectionLevel }) => {
  return (
    <div className="report-block page-break">
      <h2 className={subsectionLevel ? 'subsection-title' : 'report-block-title'}>
        {num}. {data?.title}
        <TitleLabel show={true} />
      </h2>

      <table className="statistics-table">
        <thead>
          <tr>
            <th className="table-heading" />
            <th>
              <SeverityChip severity="Critical" />
            </th>
            <th>
              <SeverityChip severity="Major" />
            </th>
            <th>
              <SeverityChip severity="Medium" />
            </th>
            <th>
              <SeverityChip severity="Minor" />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td className="table-heading">Fixed</td>
            <td>{data?.statistics?.fixed?.critical}</td>
            <td>{data?.statistics?.fixed?.major}</td>
            <td>{data?.statistics?.fixed?.medium}</td>
            <td>{data?.statistics?.fixed?.minor}</td>
          </tr>
          <tr>
            <td className="table-heading">Will Not Fix</td>
            <td>{data?.statistics?.not_fixed?.critical}</td>
            <td>{data?.statistics?.not_fixed?.major}</td>
            <td>{data?.statistics?.not_fixed?.medium}</td>
            <td>{data?.statistics?.not_fixed?.minor}</td>
          </tr>
        </tbody>
      </table>
    </div>
  )
}

export default StatisticsBlock
