import React from 'react'
import HeroLogo from '../images/HeroLogo.js'
import CornerLogo from '../images/CornerLogo.js'

const TitlePage = ({ project }) => {
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
              <div className="auditor-info">{project.auditor_name}</div>
              <div className="auditor-info">{project.auditor_email}</div>
            </div>
          </div>
          <div className="col-6 hero-image-block">
            <HeroLogo />
          </div>
        </div>
        <div className="project-name">{project.project_name}</div>
      </div>
    </div>
  )
}

export default TitlePage
