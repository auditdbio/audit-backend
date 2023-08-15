FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .
RUN cargo build --release
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]