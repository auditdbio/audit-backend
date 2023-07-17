FROM geekykaran/headless-chrome-node-docker:latest

WORKDIR /backend
COPY . .
RUN npm i
RUN npm run build

CMD ["npm", "start"]
