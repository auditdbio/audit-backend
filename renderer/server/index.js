import express from 'express'
import dotenv from 'dotenv'
import cors from 'cors'
import report from './routes/report.js'

dotenv.config()
const app = express()
const PORT = process.env.PORT || 3015

app.use(cors())
app.use(express.json())

app.use('/api', report)

app.use((err, req, res) => {
  res.status(500).send("Server error")
})

app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`)
})
