import { Router } from 'express'
import { generateReport } from '../controllers/report/report.js'
import { validateReport } from '../middleware/validateReport.js'

const router = Router()

router.post('/generate-report', validateReport, generateReport)

export default router
