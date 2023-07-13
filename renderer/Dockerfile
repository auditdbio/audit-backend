FROM node:16.17.0-bullseye-slim

WORKDIR /backend
COPY . .
RUN chmod +x ./setup.sh
RUN cat pakage.json
CMD ["./setup.sh"]
