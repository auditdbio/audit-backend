FROM rust:latest as build

WORKDIR /usr/src/audit_backend/
COPY . .

RUN cargo build --release

FROM debian:bullseye as users

COPY --from=build /usr/src/audit_backend/target/release/users /usr/local/bin/users

WORKDIR /usr/local/bin
CMD ["users"]


FROM debian:bullseye as swagger

COPY --from=build /usr/src/audit_backend/target/release/swagger /usr/local/bin/swagger

WORKDIR /usr/local/bin
CMD ["swagger"]



FROM debian:bullseye as customers

COPY --from=build /usr/src/audit_backend/target/release/customers /usr/local/bin/customers

WORKDIR /usr/local/bin
CMD ["customers"]



FROM debian:bullseye as audits

COPY --from=build /usr/src/audit_backend/target/release/audits /usr/local/bin/audits

WORKDIR /usr/local/bin
CMD ["audits"]


FROM debian:bullseye as auditors

COPY --from=build /usr/src/audit_backend/target/release/auditors /usr/local/bin/auditors

WORKDIR /usr/local/bin
CMD ["auditors"]
