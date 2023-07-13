FROM node:16.17.0-bullseye-slim

WORKDIR /backend
COPY . .
RUN npm i
RUN npm run build
RUN chmod +x setup.sh
CMD ["setup.sh"]
