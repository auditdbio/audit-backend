
FROM rust:bookworm  as chef
RUN cargo install cargo-chef --locked
WORKDIR /app
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --features test_server
RUN chmod +x ./setup.sh
CMD ["./setup.sh"]
