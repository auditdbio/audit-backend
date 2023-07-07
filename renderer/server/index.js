import express from 'express'
import dotenv from 'dotenv'
import report from './routes/report.js'

dotenv.config()
const app = express()
const PORT = process.env.PORT || 8080

app.use(express.json())

app.use('/api', report)

app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`)
})
