FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .

RUN cargo build --release


FROM debian:bullseye as swagger

COPY --from=build /usr/src/audit_backend/target/release/swagger /usr/local/bin/swagger

WORKDIR /usr/local/bin
CMD ["swagger"]

