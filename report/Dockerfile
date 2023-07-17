FROM debian:bullseye

RUN apt-get update
RUN apt-get install ca-certificates -y
WORKDIR /backend
COPY ./setup.sh .
RUN chmod +x ./setup.
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
CMD ["./setup.sh"]
