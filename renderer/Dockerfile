FROM node:latest

WORKDIR /puppeteer

COPY . .

RUN npm i
RUN npm run build

CMD ["npm", "start"]
