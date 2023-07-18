FROM node:latest

WORKDIR /puppeteer

COPY . .
RUN apt update -y
RUN apt install -y libnss3
RUN npm i
RUN npm run build

CMD ["npm", "start"]
