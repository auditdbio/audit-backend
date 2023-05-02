FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .
RUN cargo build --release --features=test_server
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]