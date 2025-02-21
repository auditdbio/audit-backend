import Joi from 'joi'

// Schemas for specific block types
const issueDataSchema = Joi.object({
  severity: Joi.string().valid('Critical', 'Major', 'Medium', 'Minor').required(),
  links: Joi.array().items(Joi.string().uri()).optional(),
  category: Joi.string().allow('').optional(),
  status: Joi.string().valid('Fixed', 'WillNotFix', 'NotFixed').required(),
  file: Joi.string().allow('').optional(),
})

const statisticsItemSchema = Joi.object({
  critical: Joi.number().min(0).required(),
  major: Joi.number().min(0).required(),
  medium: Joi.number().min(0).required(),
  minor: Joi.number().min(0).required(),
})

const statisticsSchema = Joi.object({
  fixed: statisticsItemSchema.required(),
  not_fixed: statisticsItemSchema.required(),
})

// Recursive block schema
const blockSchema = Joi.object({
  type: Joi.string().valid('markdown', 'plain_text', 'issue_data', 'statistics', 'scope', 'project_description').required(),
  title: Joi.string().required(),
  text: Joi.string().allow('').optional(),
  // Use ref for recursive schema
  subsections: Joi.array().items(Joi.link('#block')).optional(),
  // Additional fields based on type
  links: Joi.when('type', {
    is: 'scope',
    then: Joi.array().items(Joi.string().uri()).required(),
    otherwise: Joi.forbidden()
  }),
  statistics: Joi.when('type', {
    is: 'statistics',
    then: statisticsSchema.required(),
    otherwise: Joi.forbidden()
  }),
  issue_data: Joi.when('type', {
    is: 'issue_data',
    then: issueDataSchema.required(),
    otherwise: Joi.forbidden()
  }),
  feedback: Joi.when('type', {
    is: 'issue_data',
    then: Joi.string().allow('').optional(),
    otherwise: Joi.forbidden()
  })
}).id('block')

// Main request schema
export const reportRequestSchema = Joi.object({
  project_name: Joi.string().required(),
  auditor_name: Joi.string().allow('').optional(),
  profile_link: Joi.string().uri().optional(),
  audit_link: Joi.string().uri().optional(),
  report_data: Joi.array().items(blockSchema).min(1).required()
})
