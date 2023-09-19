FROM debian:bullseye

RUN apt-get update
RUN apt-get install openssl ca-certificates -y
WORKDIR /backend
COPY ./setup.sh .
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
