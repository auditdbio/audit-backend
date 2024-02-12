import React from 'react'

const SeverityChip = ({ severity }) => {
  const defineColor = () => {
    switch (severity) {
      case 'Critical':
        return '#FF0000'
      case 'Major':
        return '#FF9900'
      case 'Medium':
        return '#5b97bb'
      case 'Minor':
        return '#09C010'
      default:
        return '#434242'
    }
  }

  return (
    <div className="severity-chip" style={{ background: defineColor(severity) }}>
      {severity}
    </div>
  )
}

export default SeverityChip
