FROM rust:1.70-slim-bookworm AS builder
WORKDIR /usr/src/cathedral
COPY . .
RUN cargo install --path ./cathedral
RUN cargo install --path ./fascination

FROM debian:bookworm-slim
LABEL maintainer="kb10uy"
COPY --from=builder /usr/local/cargo/bin/cathedral /usr/local/bin/cathedral
COPY --from=builder /usr/local/cargo/bin/fascination /usr/local/bin/fascination

CMD ["/usr/local/bin/fascination"]
