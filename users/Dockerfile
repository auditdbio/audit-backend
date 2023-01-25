FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .

RUN cargo build --release

FROM debian:bullseye as users

COPY --from=build /usr/src/audit_backend/target/release/users /usr/local/bin/users

WORKDIR /usr/local/bin
CMD ["users"]
