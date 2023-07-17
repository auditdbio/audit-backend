FROM ghcr.io/puppeteer/puppeteer:latest

WORKDIR /backend
COPY . .
RUN npm i
RUN npm run build

CMD ["npm", "start"]
