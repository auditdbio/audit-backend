FROM debian:bookworm-slim

RUN apt-get update
RUN apt-get install openssl ca-certificates -y
RUN apt-get install wget -y
WORKDIR /backend
COPY ./setup.sh .
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
