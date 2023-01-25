FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY .. .

RUN cargo build --release

FROM debian:bullseye as auditors

COPY --from=build /usr/src/audit_backend/target/release/auditors /usr/local/bin/auditors

WORKDIR /usr/local/bin
CMD ["auditors"]
