import { reportRequestSchema } from '../validators/reportSchema.js'

export const validateReport = async (req, res, next) => {
  try {
    const validated = await reportRequestSchema.validateAsync(req.body, {
      abortEarly: false,
      stripUnknown: true
    })
    req.body = validated // Replace request body with validated data
    next()
  } catch (err) {
    if (err.isJoi) {
      return res.status(400).json({
        error: 'Validation Error',
        details: err.details.map(detail => ({
          path: detail.path,
          message: detail.message
        }))
      })
    }
    next(err)
  }
}
