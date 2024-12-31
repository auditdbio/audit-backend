import React from 'react'
import QRCode from 'qrcode.react'
import HeroLogo from '../images/HeroLogo.js'
import CornerLogo from '../images/CornerLogo.js'
import { FRONTEND, PROTOCOL } from '../../constants/reportLink.js'
import { logoQRbase64 } from "../images/LogoQRbase64.js"

const TitlePage = ({ project }) => {
  const titleSize = project?.project_name?.length <= 100 ? '60px' : '50px'
  const profile_link = project?.profile_link || `${PROTOCOL}://${FRONTEND}/disclaimer/`
  const audit_link = project?.audit_link || `${PROTOCOL}://${FRONTEND}/disclaimer/`

  return (
    <div className="container cover-page">
      <div className="cover-page-corner-logo">
        <CornerLogo />
      </div>
      <div className="hero">
        <div className="row">
          <div className="col-6 hero-text-block">
            <div className="hero-text">Smart Contract Security Audit Report</div>
            <div className="auditor-info-block">
              <div className="auditor-info auditor-info-heading">By</div>
              <div>
                <a className="auditor-info" href={profile_link}>
                  {project?.auditor_name}
                </a>
              </div>
              <div className="QR-wrapper">
                <QRCode.QRCodeSVG
                  value={audit_link}
                  level="H"
                  imageSettings={{
                    src: logoQRbase64,
                    height: 45,
                    width: 45,
                  }}
                />
              </div>
              <div className="verify-button">
                Verify
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
