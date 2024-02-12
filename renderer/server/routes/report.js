import { Router } from 'express'
import { generateReport } from '../controllers/report/report.js'

const router = Router()

router.post('/generate-report', generateReport)

export default router
