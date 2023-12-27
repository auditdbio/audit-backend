import dotenv from 'dotenv'

dotenv.config()

export const PROTOCOL = process.env.PROTOCOL || 'https';
export const FRONTEND = process.env.FRONTEND;
