FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .

RUN cargo build --release


FROM debian:bullseye as customers

COPY --from=build /usr/src/audit_backend/target/release/customers /usr/local/bin/customers

WORKDIR /usr/local/bin
CMD ["customers"]
