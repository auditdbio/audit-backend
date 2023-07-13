FROM node:16.17.0-bullseye-slim

WORKDIR /backend
COPY . .
RUN npm i
RUN npm run build
CMD ["npm run start"]
