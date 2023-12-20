import React from 'react'
import QRCode from 'qrcode.react'
import HeroLogo from '../images/HeroLogo.js'
import CornerLogo from '../images/CornerLogo.js'

const TitlePage = ({ project }) => {
  const titleSize = project?.project_name?.length <= 100 ? '60px' : '50px'
  const link = project?.profile_link || 'https://auditdb.io/disclaimer/'

  return (
    <div className="container">
      <div className="cover-page-corner-logo">
        <CornerLogo />
      </div>
      <div className="hero">
        <div className="row">
          <div className="col-6 hero-text-block">
            <div className="hero-text">Smart Contract Security Audit Report</div>
            <div className="auditor-info-block">
              <div className="auditor-info auditor-info-heading">By</div>
              <div className="auditor-info">{project?.auditor_name}</div>
              <div className="QR-wrapper">
                <QRCode.QRCodeSVG value={link} />
              </div>
              <div>
                <a className="auditor-info" href={link}>
                  {project?.profile_link ? "Profile link" : "auditdb.io"}
                </a>
              </div>
            </div>
          </div>
          <div className="col-6 hero-image-block">
            <HeroLogo />
          </div>
        </div>
        <div className="project-name" style={{ fontSize: titleSize }}>
          {project?.project_name}
        </div>
      </div>
    </div>
  )
}

export default TitlePage
