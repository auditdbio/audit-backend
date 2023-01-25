FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .

RUN cargo build --release


FROM debian:bullseye as audits

COPY --from=build /usr/src/audit_backend/target/release/audits /usr/local/bin/audits

WORKDIR /usr/local/bin
CMD ["audits"]

